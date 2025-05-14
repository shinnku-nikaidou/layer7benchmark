# Layer7Benchmark

[中文说明](https://github.com/shinnku-nikaidou/layer7benchmark/blob/main/README_zh.md)

## Usage

```bash
./layer7benchmark -c [concurrent-count] -a [url] -t [time]

-c or --concurrent-count u16
    concurrent thread count for download (default is 2)
-u or --url string
    url to download (default is https://www.google.com)
-t or --time u32
    time to download (default is 60)
-h or --help
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

## Introduction

Layer7Benchmark is a tool to benchmark the performance of Layer 7 (application layer) protocols, such as HTTP and HTTPS. It allows you to test the response time and throughput of your web applications under different load conditions.

It is designed to be simple and easy to use, making it suitable for both developers and system administrators who want to ensure their applications can handle the expected traffic.

DO NOT USE THIS TOOL FOR MALICIOUS PURPOSES. IT IS INTENDED FOR LEGITIMATE PERFORMANCE TESTING ONLY. USING THIS TOOL TO ATTACK OR DISRUPT WEBSITES (such as DDOS) WITHOUT PERMISSION IS ILLEGAL AND UNETHICAL.

I am not responsible for any misuse of this tool. Please use it responsibly and ethically.

## Features

This tool is completely written in Rust and is designed to be fast and efficient. It uses the `reqwest` library for making HTTP and HTTPS requests, and the `tokio` library for asynchronous programming.
So it is extremely fast and efficient. Much lower CPU usage than other tools like [webbenchmark](https://github.com/maintell/webBenchmark).

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
