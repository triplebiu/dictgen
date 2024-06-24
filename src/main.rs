use std::collections::HashSet;
use std::{fs, io};
use std::fs::File;
use std::io::Write;
use std::path::{ PathBuf};
use std::process::exit;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use clap::{ Parser};
use clap::ArgAction::*;
use itertools::Itertools;
use regex::Regex;

#[derive(Deserialize, Serialize, Debug)]
struct Element {
    code: String,
    item: Vec<String>
}
#[derive(Deserialize, Serialize, Debug)]
struct LengthFilter {
    min: usize,
    max: usize
}

/// DictGen Config
#[derive(Deserialize, Serialize, Debug)]
struct DgConfig {
    element: Vec<Element>,
    rule: Vec<String>,
    filter: LengthFilter,
    output: Option<PathBuf>
}

impl DgConfig {
    fn get_element_item(&self, code: &str) -> Option<Vec<String>> {
        for item in &self.element {
            if item.code == code {
                return Some(item.item.clone())
            }
        }
        return None
    }
}

const CONFIG_EXAMPLE: &str = r##"
# 输出的文件路径，优先级低于命令行输入，配置异常时直接输出到Stdout。
output = "dict_output.txt"

# 字典生成的规则，对应各元素项，无法匹配的字符/串将原样输出。
rule = [
    "{org}{suborg}{app}{user}{hfstr}",
    "{user}{hfstr}{hfstr}{hfstr}",
    "{org}@2024"
]

# 只输出满足长度要求的字符串。
[filter]
min = 0
max = 24

# 元素代码可自定义(数字字母)，规则中对应修改即可。
[[element]]
code = "org"
item = ["baidu", "BD"]

[[element]]
code = "suborg"
item = ["youdao", "netdisk"]

[[element]]
code = "app"
item = ["gitlab", "OA", "ERP"]

[[element]]
code = "user"
item = ["admin", "root", "guest"]

[[element]]
code = "hfstr"
item = ["123", "111", "456", "abc", "zxc", "asd", "qwe", "789", "0", "1","!","@","#","$","!@#$","."]

[[element]]
code = "C1"
item = []
"##;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
    /// Config to generate dict
    #[clap(short = 'c', value_name = "FILE", conflicts_with="example") ]
    config: Option<PathBuf>,

    /// Output file
    #[clap(short = 'o', value_name = "FILE")]
    output: Option<PathBuf>,

    /// Force overwrite target file
    ///
    /// Overwrite target file while example config file or output file
    #[clap(short = 'f', long="force",action=SetTrue)]
    overwrite: bool,

    /// Generate an example config file
    #[clap(short = 'g', value_name = "FILE")]
    example: Option<PathBuf>,
}

fn main() {
    let start = Instant::now();
    let cli: Cli = Cli::parse();

    if cli.config.is_some() {
        let configfile = cli.config.unwrap();
        if !configfile.exists() {
            println!("File {} is NOT exist. ",configfile.to_string_lossy());
            exit(1)
        } else {
            let c = fs::read_to_string(configfile).expect("Failed to read config");
            let mut conf:DgConfig = toml::from_str(&c).expect("Failed to parse toml format config");
            if cli.output.is_some() {
                conf.output = cli.output;
            }

            if conf.output.clone().unwrap_or(PathBuf::new()).exists() && !cli.overwrite {
                println!("Output file ({}) already exist.",conf.output.unwrap_or(PathBuf::new()).display());
                exit(2)
            }

            // 元素出重
            let eles = HashSet::<String>::new();
            for el in &mut conf.element {
                if eles.contains(&el.code) {
                    println!("Attention! 'code' for 'element' (code={}) is duplicated", &el.code);
                }
                el.item.push("".to_string());
                // 元素出重
                let tmp:HashSet<_> = el.item.drain(..).collect();
                // println!("{:?}\t{:?}",el.item,tmp);
                el.item.extend(tmp.into_iter());
            }

            // 规则集处理   忽略

            // println!("{:#?}",conf);
            generate_dict(conf);
        }
    }

    if cli.example.is_some(){
        let examplefile = cli.example.unwrap();
        if examplefile.exists() && !cli.overwrite {
            println!("Example file {} already exist.",examplefile.to_string_lossy());
            exit(3);
        } else {
            create_example(examplefile);
        }
    }
    let duration = start.elapsed();
    println!("{:?}",duration);
    return;
}

fn generate_dict(conf: DgConfig) {
    // println!("{:?}",conf);
    let mut alldict:HashSet<String> = HashSet::new();

    let mut ecodelist:Vec<String> = vec![];

    let ecodere = Regex::new(r"^\w+$").unwrap();
    for e in &conf.element {
        if ecodere.is_match(&*e.code) {
            ecodelist.push(e.code.clone());
        }
    }
    let elere = ecodelist.join("|");
    let elere = format!("\\{{({})\\}}",elere);

    // println!("Regex: {}",elere);

    let re = Regex::new(&*elere).unwrap();


    let mut showprocess = true;
    let mut outputfile:Box<dyn Write> = match File::create(conf.output.clone().unwrap_or(PathBuf::new())) {
        Ok(out) => {
            Box::new(out)
        }
        Err(_) => {
            showprocess = false;
            Box::new(io::stdout())
        }
    };

    for rule_item in conf.rule.iter() {
        let mut out = vec!["".to_string()];
        let mut rule2 = rule_item.to_string();
        let mut chip ;

        if showprocess {
            println!("Process rule: {}",rule_item);
        }

        for (full, [filed]) in re.captures_iter(rule_item).map(|c| c.extract()) {
            let rchip:Vec<&str> = rule2.splitn(2,full).collect();

            if rchip.len()==2 {
                chip = rchip[0].to_string();
                rule2 = rchip[1].to_string();
            } else {
                chip = "".to_string();
                rule2 = "".to_string();
            }
            out = out.iter().cartesian_product(conf.get_element_item(filed).unwrap_or(vec!["".to_string()]))
                .map(|(a, b)| format!("{}{}{}", a,chip, b)).collect();

        }

        _ = out.iter()
            .map(|x|format!("{}{}\n",x,rule2))
            .map(|x| if x.len()>=conf.filter.min && x.len()<=conf.filter.max {
                if alldict.insert(x.clone()) {
                    outputfile.write(x.as_ref()).expect("Failed to write.");
                }
            }).collect::<Vec<_>>();
    }
    let _ = outputfile.flush();
    println!("\n\tDONE!\tGenerated: {}\n{}", alldict.len(),
             if showprocess { format!("\t{}\n", conf.output.unwrap_or(PathBuf::new()).display()) } else { "".to_string() }
    )
}

fn create_example(f: PathBuf) {
    let mut file = File::create(f.clone()).expect(&format!("Failed to create file: {}",f.as_path().display()));
    file.write_all(CONFIG_EXAMPLE.as_ref()).expect(&format!("Failed to write file: {}", f.as_path().display()));
    println!("Example config file generated: {}\n{}",fs::canonicalize(f).unwrap().display(),CONFIG_EXAMPLE.to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use regex::Regex;
    use crate::{DgConfig, Element, LengthFilter};

    #[test]
    fn regex_1() {
        let ecodere = Regex::new(r"^\w+$").unwrap();


            assert!(ecodere.is_match("123"));

        let mut restr = r"\{(org|sorg|app|user|hfstr)\}";
        let re = Regex::new(restr).unwrap();

        let a = re.is_match("bad{app}xxx");
        println!("{}",a);

    }

    #[test]
    fn conftest() {
        let c = DgConfig {
            element: vec![
                Element { code: "E1".to_string(), item: vec!["abc".to_string(), "zxc".to_string()] },
                Element { code: "E2".to_string(), item: vec!["admin".to_string(),"root".to_string()] },
                Element{ code: "E3".to_string(), item: vec!["123".to_string(),"111".to_string()] }
            ],
            rule: vec!["{E1}{E2}{E3}".to_string(),"{E2}{E3}{E3}".to_string()],
            filter: LengthFilter { min: 0, max: 12 },
            output: Option::from(PathBuf::from("dict_output.txt")),
        };

        let toml = toml::to_string(&c).unwrap();

        println!("{:#?}",toml);
    }

}
