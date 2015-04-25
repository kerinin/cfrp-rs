#!/bin/bash

set -o errexit -o nounset

rev=$(git rev-parse --short HEAD)

cd target/doc/cfrp

git init
git config user.name "Ryan Michael"
git config user.email "kerinin@gmail.com"

git remote add upstream "https://$GH_TOKEN@github.com/kerinin/cfrp-rs.git"
git fetch upstream
git reset upstream/gh-pages

# echo "rustbyexample.com" > CNAME

touch .

git add -A .
git commit -m "rebuild docs at ${rev}"
git push -q upstream HEAD:gh-pages
