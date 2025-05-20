# Layer7Benchmark

[中文说明](https://github.com/shinnku-nikaidou/layer7benchmark/blob/main/README_zh.md)

## Introduction

Layer7Benchmark is a tool to benchmark the performance of Layer 7 (application layer) protocols, such as HTTP and HTTPS. It allows you to test the response time and throughput of your web applications under different load conditions.

It is designed to be simple and easy to use, making it suitable for both developers and system administrators who want to ensure their applications can handle the expected traffic.

DO NOT USE THIS TOOL FOR MALICIOUS PURPOSES. IT IS INTENDED FOR LEGITIMATE PERFORMANCE TESTING ONLY. USING THIS TOOL TO ATTACK OR DISRUPT WEBSITES (such as DDOS) WITHOUT PERMISSION IS ILLEGAL AND UNETHICAL.

I am not responsible for any misuse of this tool. Please use it responsibly and ethically.

## Features

This tool is completely written in Rust and is designed to be fast and efficient. It uses the `reqwest` library for making HTTP and HTTPS requests, and the `tokio` library for asynchronous programming.
So it is extremely fast and efficient. Much lower CPU usage than other tools like [webbenchmark](https://github.com/maintell/webBenchmark).

## Usage

```bash
./layer7benchmark -c [concurrent-count] -a [url] -t [time]

-c or --concurrent-count u16
    concurrent thread count for download (default is 2)
-u or --url string
    url to download (default is https://www.google.com)
-t or --time u32
    time to download (default is 60)
--help
    help for layer7benchmark
-v or --version
    version for layer7benchmark
-H or --header string
    http header to send (-H is exactly the same as curl command in order to be compatible with it)
-i or --ip string
    ip to send the request to (default is automatically resolved from the url)
    if you have already found the original ip address, you can use this option to bypass the CDN or some random WAF
--test
    test mode, only send one request for testing or debugging the answer
-X or --method string
    http method to use (default is GET) options: GET or POST or PUT or DELETE or OPTIONS (also -X is still exactly the same as curl command)
--timeout u64
    timeout for the request (default is 10)
    If the request takes longer than this time, it will be considered a timeout
    It is different from the get stream timeout, which is applied to the full request body,
    which in that case the time out is set to 60 seconds, different from the request timeout
--body string
    body to send with the request (default is empty)
    This option is only valid for POST and PUT requests
    You could add this option in GET requests, but it will be ignored
--random
    ⚠️ If you use this option, the --url option grammar will be changed.
    In summary, this program will now randomly generate URLs based on your --url option.
    For example, if you set --url to https://www.example.com/[a-z0-9]{10},
    the program will randomly generate URLs like https://www.example.com/abc123xyz0,
    https://www.example.com/xyz789abc1 , etc., and send requests to these random URLs.
    If you want to use this option, please make sure you understand the grammar of the URL you set.
    This option can be combined with the --test option. And the --test option will only send one request to a randomly generated URL, also the --test option will print the URL which is randomly generated.
    Full grammar is down below and you can keep reading the following text.
--ip-files
    Some times a website will have multiple IP addresses when using a CDN or load balancing.
    You can use this option to specify a file containing a list of IP addresses, one per line.
    The program will randomly select one of the IP addresses from the file for each request.
    This option conflicts with the --ip option.  And please provide valid file path, and the contents of the file must be valid IP addresses' list.
```

### Example

```bash
./layer7benchmark -u https://www.example.com --test

./layer7benchmark -u https://www.example.com -t 60 \
    --header "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3" \
    --header "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8" \
    --header "Accept-Language: en-US,en;q=0.5" \
    --header "Accept-Encoding: gzip, deflate, br" \
    --header "Connection: keep-alive" \
    --header "Cache-Control: max-age=0"

./layer7benchmark -u https://x.com/home -c 16 -t 360 --ip 172.66.0.227
```

## random URL grammar

In summary, this program will now randomly generate URLs based on your `--url` or `-u` option.
There are two types of grammar.

1. First is `[expr]`
2. Second is `[expr]{n}`

`expr` contains three types of characters: `a-z`, `A-Z`, `0-9`. You can combine them together.
such as `[a-zA-Z0-9]`, `[a-zA-Z]`, `[0-9]`, etc.

> note: `[a-zA-Z0-9]` will generate a random character from `a-z`, `A-Z`, `0-9`.

`n` is a number, which means how many characters you want to generate.
such as `[0-9]{3}` will generate 3 random characters from `0-9`. such as `111`, `456`, `364`, etc.

> note: `[a-zA-Z]` is exactly the same as `[a-zA-Z]{1}` or [A-Za-z].

### some examples

```bash
https://www.example.com/[0-9]{3}

# will generate a random URL like
https://www.example.com/111
https://www.example.com/456
https://www.example.com/364

https://www.example.com/random/path?foo=[a-z0-9]&bar=[a-zA-Z]{3}
# will generate a random URL like
https://www.example.com/random/path?foo=1&bar=MPL
https://www.example.com/random/path?foo=2&bar=deD
https://www.example.com/random/path?foo=a&bar=gHi

```

## Build

```bash
# only for Linux x86_64
# install protobuf
wget https://github.com/protocolbuffers/protobuf/releases/download/v31.0/protoc-31.0-linux-x86_64.zip
unzip protoc-31.0-linux-x86_64.zip -d protoc31
sudo cp protoc31/bin/protoc /usr/local/bin/
sudo cp -r protoc31/include/* /usr/local/include/

# install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# build
git clone https://github.com/shinnku-nikaidou/layer7benchmark.git
cd layer7benchmark
cargo build --release
```

## 中文说明

### 简介

Layer7Benchmark 是一个用于基准测试 Layer 7（应用层）协议（如 HTTP 和 HTTPS）性能的工具。它允许您在不同负载条件下测试 Web 应用程序的响应时间和吞吐量。
它旨在简单易用，适合开发人员和系统管理员，确保其应用程序可以处理预期的流量。
它完全用 Rust 编写，旨在快速高效。它使用 `reqwest` 库进行 HTTP 和 HTTPS 请求，并使用 `tokio` 库进行异步编程。
因此，它非常快速高效。比其他工具（如 [webbenchmark](https://github.com/maintell/webBenchmark)）的 CPU 使用率低得多。

- 不要使用这个工具进行恶意目的。它仅用于合法的性能测试。未经许可使用此工具攻击或破坏网站（例如 DDOS）是非法和不道德的。
- 我不对此工具的任何误用负责。请负责任和道德地使用它。

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/shinnku-nikaidou/layer7benchmark/blob/main/License) file for details.
