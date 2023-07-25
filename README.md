**Work In Progress**

**Not ready yet**

🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑🛑

[![codecov](https://codecov.io/gh/aws-cli-tools/chaos-lambda-extension/branch/main/graph/badge.svg)](https://codecov.io/gh/aws-cli-tools/chaos-lambda-extension)
[![Actions Status](https://github.com/aws-cli-tools/chaos-lambda-extension/workflows/Code%20Gating/badge.svg?branch=main)](https://github.com/aws-cli-tools/chaos-lambda-extension/workflows/Code%20Gating/badge.svg?branch=main)

# lambda-chaos-extension

Using `lambda-chaos-extension` to inject faults to Lambda functions without any modification to function code.
Unlike previous chaos implementation that required tight coupling with the Lambda runtime, this extension is agnostic to the runtime, and can run on any runtime that utilizes Amazon Linux 2 under the hood.
Right now the following runtimes are supported:
* Node.js 18
* Node.js 16
* Node.js 14
* Python 3.10
* Python 3.9
* Python 3.8
* Java 17
* Java 11
* Java 8
* .NET 6
* Ruby 3.2
* Ruby 2.7
* provided.al2
 

This extension inject two faults depending on configuration: 

1. Add latency.
2. Replace function response.

## Configuration
The extension is controled via environment variables
### Latency fault
* `CHAOS_EXTENSION__LAMBDA__ENABLE_LATENCY` - Enable latency fault injection. Set to either `true` or `false`. Default is `false`.
* `CHAOS_EXTENSION__LAMBDA__LATENCY_VALUE` - How much latency to add to the lambda. Value in seconds. Default is `900` seconds.
* `CHAOS_EXTENSION__LAMBDA__LATENCY_PROBABILITY` - A number between 0 to 1 that determined the probability of injecting the fault. Default it `0.9`

### Response fault
* `CHAOS_EXTENSION__RESPONSE__ENABLE_CHANGE_REPONSE_BODY` - Enable response fault injection. Set to either `true` or `false`. Default is `false`.
* `CHAOS_EXTENSION__RESPONSE__DEFAULT_RESPONSE` - The response to return as a stringified json. Default is 
```json
{
    "statusCode": 500,
    "body": {
        "message": "hello, Chaos!!!"
    }
}
```
* `CHAOS_EXTENSION__RESPONSE__CHANGE_RESPONSE_PROBABILITY` - A number between 0 to 1 that determined the probability of injecting the fault. Default it `0.9`

## Deployment
### Public 
The chaos extension is publicly available. The latest versions of the layer are available in [LAYERS.md](LAYERS.ms)
You can use the extension using the console or your preferred IAC solution. You can see how to do it in AWS SAM in the `example` folder example.
 

## Running locally
* You can always use `cargo` to manage the build and tests.
* We use [`just`](https://github.com/casey/just) as a command running.
* Use `just gate` to run all checks locally.

## Contributing
See our [CONTRIBUTION](CONTRIBUTION.md) page
