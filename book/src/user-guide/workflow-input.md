# Workflow input

In the examples so far, usually the `start` function generates some "input" to be processed by the workflow.

In this chapter, we'll see how to pass input to a workflow.

## Passing simple input

To pass input to a workflow, you can use the `--input` flag (or `-i` for short) when running the workflow.

For example, to pass the number `42` to the `start` function of a workflow deployed under the name `my-deployment-name`, you would run:

```bash
florca run -d "my-deployment-name" --input 42
```

The input is then available in the `payload` property of the `requestBody` object passed to the entry-point function:

```typescript
import type { PluginRequestBody, ResponseBody } from "@florca/fn";

export default async (
  requestBody: PluginRequestBody
): Promise<ResponseBody> => {
  const input = requestBody.payload as number; // input is 42
  return {
    payload: input,
  };
};
```

## Passing complex input

The CLI parses the input as JSON.

For strings, this means you need to wrap them in quotes, like so:

```bash
florca run -d "my-deployment-name" --input '"Hello, world!"'
```

This will pass the string `Hello, world!` to the workflow.

For more complex input, you can pass JSON objects:

```bash
florca run -d "my-deployment-name" --input '{"name": "Alice", "age": 42}'
```

## Accessing complex input

While not necessary, you can cast the input to a TypeScript type to make it easier to work with:

```typescript
import type { PluginRequestBody, ResponseBody } from "@florca/fn";

type Input = {
  name: string;
  age: number;
};

export default async (
  requestBody: PluginRequestBody
): Promise<ResponseBody> => {
  const input = requestBody.payload as Input;
  return {
    payload: {},
  };
};
```

### Non-plugin functions

If it's not a TypeScript plugin function you're dealing with, but a Python AWS Lambda function, for example, you could define a `TypedDict` or a `dataclass` to represent the input:

```python
from typing import TypedDict
class Input(TypedDict):
    name: str
    age: int
```

```python
from dataclasses import dataclass
@dataclass
class Input:
    name: str
    age: int
```

_Since we haven't covered AWS Lambda functions yet, we won't go into more detail here. Just keep in mind that not only plugin functions can act as entry points and receive input, but also remote functions. See the [AWS Lambda functions](aws-lambda-functions.md) chapter for more information._
