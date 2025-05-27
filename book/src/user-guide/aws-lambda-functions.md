# AWS Lambda functions

This guide will show you how to incorporate AWS Lambda functions into your workflows.

<div class="warning">

**Be aware of potential costs when using AWS Lambda functions!**

The AWS Lambda functions are deployed to your AWS account and will incur costs based on the number of invocations and the duration of the function execution. Make sure to monitor your AWS account for any unexpected charges and set up billing alerts if necessary.

Specifically, watch out for potential infinite loops in your workflows, as they could lead to high costs.

Of course, there is also the eventuality of bugs in the engine or the deployer that could lead to unexpected charges.

</div>

## Configuration

<div class="warning">

The exact steps will only be covered in a future version of this guide.

For now, you can follow these steps to get started:

</div>

- Log in to the [AWS Management Console](https://aws.amazon.com/console/)
- Create an IAM user with the `AWSLambda_FullAccess` policy attached
- Create an access key for the user
- With the access key and secret key at hand, either ...
  - Run `aws configure` to [configure the AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-authentication-user.html#cli-authentication-user-configure-wizard) if you have it [installed](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html)
    - _The deployer and engine services will pick up the credentials from the default profile_
  - Or set `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` in the `.env` file
- Create an IAM role with the `AWSLambdaBasicExecutionRole` policy attached and note down the ARN of the role
- Uncomment the `AWS_ROLE` line in the `.env` file and enter the ARN of the IAM role just created
- Run `docker compose up -d` so that the deployer and engine services can pick up the change made to the `.env` file

### Override environment variables

If you'd like to move certain parts of `.env` to a separate file, you can do so by creating a `.env.local` file and then run `docker compose` with both `.env` (loaded if none is specified) and `.env.local`:

```bash
docker compose --env-file .env --env-file .env.local up -d
```

_The latter file will override the values in the former._

## Create a new workflow

Let's start off by generating a workflow with a few functions:

```bash
florca new -w workflows/aws-example plugin "start"
florca new -w workflows/aws-example function -p aws -r "python3.13" "plusOne"
florca new -w workflows/aws-example plugin "double"
```

This will create a workflow directory `workflows/aws-example` with the following functions:

- `start` (plugin)
- `plusOne` (AWS Lambda function using the `python3.13` runtime)
- `double` (plugin)

<div class="warning">

Note that an AWS Lambda function could also be first (the entry point, by convention called `start`) or last in the workflow. In this example, we chose to place it in the middle to deal with both input and output.

</div>

### Supported runtimes

The framework does not really care about what runtime you use for your AWS Lambda functions. However, for a few select runtimes, we ship templates that make it easier to get started. Run `florca templates` to see runtimes with templates.

## Implement the functions

Next, let's chain these functions together by editing the respective files.

### `aws-example/start.ts`

```typescript
{{#include ../../../examples/aws-example/start.ts}}
```

This will just forward the number `5` to the `plusOne` function.

### `aws-example/plusOne/aws/index.py`

```python
{{#include ../../../examples/aws-example/plusOne/aws/index.py}}
```

Here, we add `1` to the input and forward it to the `double` function.

### `aws-example/double.ts`

```typescript
{{#include ../../../examples/aws-example/double.ts}}
```

The last function will just double the input and return it. Since there is no `next` in the response, the workflow will end here.

## Deploy the workflow

```bash
florca deploy -w workflows/aws-example
```

<div class="warning">

### Add a `.florcaignore` file to the workflow directory to exclude files from being submitted to the deployer

Excluding files that are not needed for the workflow to run is important for the following reasons:

- Transferring large files can take a long time.
- If the content of a function changes, the deployer will redeploy the function. Better exclude changing files that are not needed for the workflow to run.

The semantics of `.florcaignore` are essentially the same as `.gitignore`, so you should be able to use the same patterns.

</div>

## Run the workflow

```bash
florca run -d "aws-example" --wait --show-outputs
```

This will run the workflow and wait for it to finish. The output should look something like this:

```ts
Run: 98
Success: true
Output: 12
Workflow: aws-example
+ start {"next":"plusOne","payload":5}
| plusOne {"next":"double","payload":6}
| double {"payload":12}
```

## Limitations

In _FloatingOrca_, AWS Lambda functions can neither register message handlers nor expose HTTP endpoints.

Furthermore, they can not run child functions using the `context.run` method.

Also note that the engine must be publicly accessible (or at least in the same network) for AWS Lambda functions to be able to send messages to other functions.
While there should be a couple of ways to achieve this, the only one tested so far is to host the engine on a public server and let a domain name point to it.
See the [Self-hosting chapter](./self-hosting.md) for more information on how to do this.
