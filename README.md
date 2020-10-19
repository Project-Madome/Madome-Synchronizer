# Madome Synchronizer

## Usage

```bash
touch fail_store.txt
touch .token # for madome

cargo build --release

PAGE=1 PER_PAGE=25 LATENCY=3600 ./target/release/madome-synchronizer

# Support Eenvironment Variables
# * ID=uint
# - Synchronize from specified ID
#
# * RETRY_FAIL=any-value
# - Retry synchronize failed ids
#
# * INFINITY=any-value
# - Synchronize all page
#
# * PAGE=uint
# - Initial page of hitomi
#
# * PER_PAGE=uint
# - Per page of hitomi
#
# * LATENCY=secs
# - Time until next synchronize
```
