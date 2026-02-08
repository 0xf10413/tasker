# Tasker

## Definitions

A _task_ represents some action that must be performed.

It is defined by:
* a _priority_, which is a capital letter from the latin alphabet (i.e. one of {A, B, ..., Z})
* a _description_, which is an arbitrary long text
* an _completion_ state, which is either `true` (for a completed task) or `false` (for a pending task).

Internally, to ease their manipulations, tasks may also bear an _identifier_ (or _id_).

A task with no completed status is _pending_. This is the default status.

A task may be _flagged_ as _completed_ when the corresponding action is completed. Tasks have no other status.

_Flagging as pending_ is the action of reverting the status from "completed" back to "pending".

The collection of all tasks is called a _task list_.

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

Note that the priority of completed tasks is not shown under this representation, but may be kept internally.

## Deleting tasks

Completed tasks may be _deleted_ at any time. For the sake of simplicity, this deletion is performed at the user's request. This process is called _task cleanup_ and deletes all completed tasks.

## Projects

A _project_ is a group of tasks. A project simply has a _name_ and no other attributes.

Tasks may be part of a project, or not be part of any project.

Projects do not exist outside of tasks. In other words, projects are not directly created or deleted. They merely exist iff they are mentioned by at least one task.

Projects can be _renamed_.

## Task presets

Some tasks are expected to come back periodically, like tridying up the house.

_Presets_ are collections of (quasi-)tasks that can be _injected_ into the task list.

A (quasi-)task within a preset is called a _preset task_.

Preset tasks only contain a priority and a description. In particular, they have no completion status nor do they pertain to any project.

When a preset is injected into the task list, all created tasks are associated with a project bearing the preset's name.

Due to their additional complexity, unlike projects, presets need to be _created_ first before any preset task can be _added_ to them.
