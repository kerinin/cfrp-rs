* Try to return Signal<A>, hide the rest of the implementation
* Make inputs generic (`listen`)
* Don't push to forks unless they have children
* `push_to` should take an optional target, so we can avoid running "dead end"
  code
* memoization & caching
* return termination handle from topology `run`
* implement async, liftn const, etc (liftn takes vector of signals and function
  taking vector of values)
* macro
* (more) tests
