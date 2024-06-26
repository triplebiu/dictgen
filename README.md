# dictgen

自定义字典生成工具。
线上版： https://triplebiu.github.io
### Usage
```bash
> dictgen -h
Usage: dictgen.exe [OPTIONS]

Options:
  -c <FILE>      Config to generate dict
  -o <FILE>      Output file
  -f, --force    Force overwrite target file
  -g <FILE>      Generate an example config file
  -h, --help     Print help (see more with '--help')
  -V, --version  Print version

```

### example config
```toml
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
```

### example
```bash
PS C:\Users\xx\coding\dictgen> cargo run -- -c config.toml -f
   Compiling dictgen v0.1.0 (C:\Users\xx\coding\dictgen)
    Finished dev [unoptimized + debuginfo] target(s) in 1.13s                                                                                                                                                                               
     Running `target\debug\dictgen.exe -c config.toml -f`
Process rule: {org}{suborg}{app}{user}{hfstr}{hfstr}
Process rule: {user}{hfstr}{hfstr}{hfstr}{hfstr}

        DONE!   Generated: 310423
        dict_output.txt

1.0135119s
```