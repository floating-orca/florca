# Parallel execution

In this chapter, we will explore how to run functions in parallel in two different ways:

1. Using the `map` function, which runs the same function on multiple inputs.
2. Using the `parallel` function, which runs multiple functions on the same input.

## The `map` function

The `map` function is a built-in function that runs the same function on multiple inputs in parallel and then reduces the results in a specified way.
It is useful when you have a list of inputs and want to process them concurrently.

To demonstrate the `map` function, let's begin by creating a `start` function:

```typescript
{{#include ../../../examples/double/start.ts}}
```

In this example, we have an array of numbers `[1, 2, 3, 4, 5]`. We want to double each number using a function called `double` and then sum up the results using a function called `sum`.

Next, let's implement the `double` function:

```typescript
{{#include ../../../examples/double/double.ts}}
```

The `double` function doubles the input number.

Finally, let's implement the `sum` function:

```typescript
{{#include ../../../examples/double/sum.ts}}
```

Since `sum` does not return a `next`, the workflow will end here, and the result will be the sum of the doubled numbers.

### How `map` works

Note that this time, in the `start` function, we don't pass a string to the `next` property but an object whose key represents the function to run (`map`, in this case).

Returning a `next` object instead of a string is an alternative way to specify the next function to run, which allows us to pass parameters to the function.
Parameters passed in that way are available to the next function via `requestBody.context.params`.

The `map` implementation specifically expects two properties: `fn` and `reduce`.
It's implementation is quite simple:

```typescript
{{#include ../../../packages/driver/lib/functions/map.ts}}
```

Essentially, the `map` function runs the function specified in `fn` on each input in parallel.
It then waits for all the results to be ready and passes them to the next function specified in `reduce`.

### Running the workflow

Inspecting a run of this workflow with `--show-inputs`, `--show-params`, and `--show-outputs` should yield the following output:

```ts
Run: 2
Success: true
Output: 30
Workflow: double
+ start {"next":{"map":{"fn":"double","reduce":"sum"}},"payload":[1,2,3,4,5]}
| map [1,2,3,4,5] {"fn":"double","reduce":"sum"} {"next":"sum","payload":[2,4,6,8,10]}
  + double 3 {"payload":6}
  + double 1 {"payload":2}
  + double 5 {"payload":10}
  + double 4 {"payload":8}
  + double 2 {"payload":4}
| sum [2,4,6,8,10] {"payload":30}
```

### Passing parameters to a child function

Alternatively, in `start`, instead of handing control over to `map` via `next`, we could also run `map` as a child function and await the result of that branch. Here's `startChild.ts`:

```typescript
{{#include ../../../examples/double/startChild.ts}}
```

Deploying the updated workflow and running it with `--show-inputs`, `--show-params`, `--show-outputs`, and `--entry-point startChild` should yield the same result as before, but with a different execution trace:

```ts
Run: 4
Success: true
Output: 30
Workflow: double
+ startChild {"payload":30}
  + map [1,2,3,4,5] {"fn":"double","reduce":"sum"} {"next":"sum","payload":[2,4,6,8,10]}
    + double 1 {"payload":2}
    + double 3 {"payload":6}
    + double 5 {"payload":10}
    + double 2 {"payload":4}
    + double 4 {"payload":8}
  | sum [2,4,6,8,10] {"payload":30}
```

## The `parallel` function

Another built-in function is `parallel`—useful when you process the same input with multiple functions concurrently.
Structurally, it is similar to `map`, but instead of running the same function on multiple inputs, it runs multiple functions on the same input.

Here's an example where we tell the `parallel` function to run two functions, `translateToGerman` and `translateToFrench`, on our string "Hello, world!" and finally reduce the results using a custom function `merge`:

```typescript
{{#include ../../../examples/translate/start.ts}}
```

See the `examples/translate` directory for the full implementation of this example.

### How `parallel` works

To give you an example of how simple utility functions such as `parallel` can be, here's the implementation:

```typescript
{{#include ../../../packages/driver/lib/functions/parallel.ts}}
```
