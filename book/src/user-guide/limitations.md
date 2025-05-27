# Limitations

This document lists limitations that are not mentioned elsewhere in the documentation.

## Non-goals

- Raw performance
  - We are not trying to squeeze every last bit of performance out of every operation
    - Instead, we try to keep the system's complexity low and code easy to read and maintain

## Not (yet) implemented / Future work

- Integrated dependency management for AWS Lambda functions
  - Currently, you have to fetch functions' dependencies yourself (e.g. using `pip` or `npm`) before you deploy the workflow
- Testing infrastructure for functions
