# Madome Synchronizer

## Usage

```bash
touch fails.txt
touch .token # for madome

cargo build --release

PAGE=1 PER_PAGE=25 LATENCY=3600 ./target/release/madome-synchronizer

# Support Eenvironment Variables
# * INFINITY=any-value
# - Synchronize all page
#
# * PAGE=uint
# - initial page of hitomi
#
# * PER_PAGE=uint
# - per page of hitomi
#
# * LATENCY=secs
# - Time until next synchronize
```
