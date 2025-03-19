#!/usr/bin/env bash

set -e

/bin/server migrate /migrations
exec /bin/server start
