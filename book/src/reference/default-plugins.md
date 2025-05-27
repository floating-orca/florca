# Default plugins

_FloatingOrca_ comes with a set of plugins that provide common functionality for workflows. This chapter describes these default plugins and how to use them.

## `map`

Takes a list of values, applies a mapping function to each value in parallel, and reduces the results using a specified reduction function.

### Example

```typescript
return {
  payload: [1, 2, 3, 4, 5],
  next: {
    map: {
      fn: "double",
      reduce: "sum",
    },
  },
};
```

In this example, the `map` plugin applies the `double` function to each number in the payload and then reduces the results using the `sum` function. The `sum` function thus needs to be defined to accept a list of values as input.

_See the `examples/double` workflow for a complete example._

## `parallel`

Takes a value and a list of functions, applies each function to the value in parallel, and reduces the results using a specified reduction function.

### Example

```typescript
return {
  payload: "Hello, world!",
  next: {
    parallel: {
      fns: ["translateToGerman", "translateToFrench"],
      reduce: "merge",
    },
  },
};
```

In this example, the `parallel` plugin applies the `translateToGerman` and `translateToFrench` functions to the input string in parallel and then merges the results using the `merge` function. The `merge` function thus needs to be defined to accept a list of values as input.

_See the `examples/translate` workflow for a complete example._
