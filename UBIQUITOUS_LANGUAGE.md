# Tasker

## Definitions

A _task_ represents some action that must be done.

It is defined by:
* a _priority_, which is a capital letter from the latin alphabet (i.e. one of {A, B, ..., Z})
* a _description_, which is an arbitrary long text
* an _completion_ state, which is either `true` (for a completed task) or `false` (for a pending task).

Internally, to ease their manipulations, tasks may also bear an _identifier_ (or _id_).

A task with no completed status is _pending_. A task with a completion date is _done_. Tasks have no other status.

The collection of all tasks is called a _task list_.

> TODO: add concept of context/project
> TODO: add completion date

## Working with tasks

The user can create as many tasks as they want. Only the description and priority are required.

The priority and description of a task may be changed by the user.

It is expected for tasks to be always shown by decreasing order of priority. If two tasks have the same priority, they should be shown by (decreasing) alphabetical order.

The _canonical representation_ of a task is mostly as described on [todo.txt][https://github.com/todotxt/todo.txt]. Here's a short summary:
```
# This is a pending task with "A" priority
(A) clean up laundry

# This is a pending task with "C" priority
(C) buy a new vacuum cleaner

# This is a completed task with "A" priority
x wish mom a happy birthday
```

Note that the priority of completed tasks is not shown.
