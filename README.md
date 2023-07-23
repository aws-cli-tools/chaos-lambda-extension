[![codecov](https://codecov.io/gh/aws-cli-tools/whoami/branch/main/graph/badge.svg?token=NW4955XIZT)](https://codecov.io/gh/aws-cli-tools/whoami)
[![Actions Status](https://github.com/aws-cli-tools/whoami/workflows/Code%20Gating/badge.svg?branch=main)](https://github.com/aws-cli-tools/whoami/workflows/Code%20Gating/badge.svg?branch=main)

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
The chaos extension is publicly available using `arn:aasee`. You can pull the latest version for your region easily by running
```bash
aws --region
```
You can se the extension using the console or your preferred IAC solution. You can see how to do it in AWS SAM n the `example` folder example.
 

To build and deploy your application for the first time, run the following in your shell:

```bash
sam build --use-container
sam deploy --guided
```

## Chaos Tests

Browse the API Gateway URL or curl it from command line for couple of times. 

- The normal results are status 200, {"message": "hello world"}. 
- 50% of the responses are status 500, {"message": "hello, Chaos!!!"}
- 10% of the responses are status 502, {"message": "Internal server error"}. 

## Running locally
* You can always use `cargo` to manage the build and tests.
* We use [`just`](https://github.com/casey/just) as a command running.
* Use `just gate` to run all checks locally.

## Contributing
See our [CONTRIBUTION](CONTRIBUTION.md) page
