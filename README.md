# Introduction

This project analyses or transforms a structured log from a rippled server.
Currently, rippled writes log lines like this:

```
2021-Feb-15 15:09:51.558872748 UTC LoadMonitor:WRN Job: processLedgerData run: 2ms wait: 1455ms
```

A structured log would be written like this:

```
2021-Feb-15 15:09:51.558872748 UTC LoadMonitor:WRN Job latency {"job": "processLedgerData", "run(ms)": 2, "wait(ms)": 1455, "jlogId": 115}
```

The advantage of the structured log format is it makes is easier to write
scripts that extract data from the log and transform that data in useful ways.
This project implements five simple examples for how to take advantage of the
structured log. Of course, it's possible to write such scripts without
structured logs, but the lack of structure makes it much more difficult. Note

Note that the "message followed by json data" format makes is easier to slowly
migrate old log messages to the new format. An old log message is just a
structured log message without any json data.

# Log lines histogram

The `-h <output_file>` option writes a histogram of the log file. It counts the
number of log lines it considers part of the same group, sorts them by log level
and count, and outputs the count and the first log line of type as an example.

This gives a useful quick overview of what's in the log file.

As an implementation note, it's much simpler for the program to decide when log
lines are the same when they are structured.

Here's an example snippet:

```
Warning
71 : 2021-Feb-13 22:15:20.113974252 UTC LoadMonitor:Warning Job latency
{
  "jlogId": 115,
  "job": "InboundLedger",
  "run(ms)": 0,
  "wait(ms)": 1160
}

19 : 2021-Feb-13 22:15:42.071415494 UTC LedgerConsensus:Warning Need consensus ledger
{
  "hash": "0A8AA83338E3DDD69318EE018FBF9C8572B45CEFC933C8F8908A1DED8CEC12D8",
  "jlogId": 116
}
```

That snippet shows that in the `warning` log level, the most messages are about
"Job latency" and there are 71 such messages. The next most messages are about
"Need consensus ledger" and there are 71 such message. Notice it also prints the
json data on an separate line and pretty prints the data for easier reading.

# Group by

The `-g <output_file>` option is similar to the "histogram" option, but instead
of writing just one log line, it writes all the log lines that are considered
part of the same group.

This is useful when we're interested in a particular group and want to see what
the data looks like.

Here's an example snippet:

```
>>>> Group Size: 2
2021-Feb-13 22:14:52.821294033 UTC TaggedCache:Debug cache target size is set
{
  "cacheName": "LedgerCache",
  "jlogId": 109,
  "size": 256
}
2021-Feb-13 22:14:52.821294033 UTC TaggedCache:Debug cache target size is set
{
  "cacheName": "LedgerCache",
  "jlogId": 109,
  "size": 256
}
<<<<
```

# Job latency report

The `-l <output_file>` looks at the "Job latency" log lines, groups them by job
type, and writes stats about the run time and wait time.

This is useful to understanding run-time behavior of the job.

Here's an example snippet:
```
Job: processLedgerData: Max Run: 13050 Max Wait: 12982 Ave Run: 1878.38 Ave Wait: 3633.30
Run histogram:
     0 : 7      ********************
     1 : 0      
     2 : 0      
     4 : 1      **
     8 : 0      
    16 : 0      
    32 : 1      **
    64 : 3      ********
   128 : 2      *****
   256 : 3      ********
   512 : 6      *****************
  1024 : 7      ********************
  2048 : 9      **************************
  4096 : 11     ********************************
  8192 : 9      **************************
 16384 : 1      **

Wait histogram:
     0 : 4      *****
     1 : 0      
     2 : 0      
     4 : 0      
     8 : 0      
    16 : 0      
    32 : 0      
    64 : 0      
   128 : 0      
   256 : 1      *
   512 : 0      
  1024 : 3      ****
  2048 : 14     *******************
  4096 : 12     ****************
  8192 : 23     ********************************
 16384 : 3      ****


Job: InboundLedger: Max Run: 9926 Max Wait: 12463 Ave Run: 1985.20 Ave Wait: 4863.00
Run histogram:
     0 : 4      ********************************
     1 : 0      
     2 : 0      
     4 : 0      
     8 : 0      
    16 : 0      
    32 : 0      
    64 : 0      
   128 : 0      
   256 : 0      
   512 : 0      
  1024 : 0      
  2048 : 0      
  4096 : 0      
  8192 : 0      
 16384 : 1      ********

Wait histogram:
  2048 : 2      ********************************
  4096 : 0      
  8192 : 2      ********************************
 16384 : 1      ****************

```

# Reformat as json

The `-j <output_file>` reformats the log file so each log line is a json object.
This is useful when writing quick and dirty scripts to analyze the log. For
example, `jq` could easily use this format.

Example snippet:

```json
{
  "data": {
    "cookie": 14698052816975440795,
    "jlogId": 99,
    "node": "C890E925BC571F2C80A5A4EA241A86CEE374E3AB"
  },
  "level": "Info",
  "module": "LedgerConsensus",
  "msg": "Consensus engine started",
  "timestamp": "2021-Feb-13 22:14:52.819951178 UTC"
}
{
  "data": {
    "jlogId": 1395,
    "version": "rippled-1.7.0-rc2+DEBUG"
  },
  "level": "Info",
  "module": "Application",
  "msg": "process starting",
  "timestamp": "2021-Feb-13 22:14:52.820191489 UTC"
}
```

# Reformat as json mixed

The `-m -j <output_file>` reformats the log file so the "data" part is pretty
printed and appears on a separate line. This is useful for reading log lines
that contain a lot of data.

Example snippet:

```
2021-Feb-13 22:15:15.000158015 UTC LedgerConsensus:Warning View of consensus changed
{
  "endLedger": "10A6340F4D571A9977FF03BBAAA899ED7AC67E226ED6BEFEA42EF53BFB6CEADB",
  "jlogId": 1069,
  "mode": "wrongLedger",
  "phase": "establish",
  "prevLedger": {
    "accepted": true,
    "account_hash": "719A6C8FF54A6EC6075DF412397853ED977F2E9A34D25E63A4ABA14ADE496BCA",
    "close_flags": 0,
    "close_time": 666569710,
    "close_time_human": "2021-Feb-13 22:15:10.000000000 UTC",
    "close_time_resolution": 10,
    "closed": true,
    "hash": "559101AB32E73563518AF6A593849E13715D4BC03AF7B37BEB1BFDCEBA6F4C7D",
    "ledger_hash": "559101AB32E73563518AF6A593849E13715D4BC03AF7B37BEB1BFDCEBA6F4C7D",
    "ledger_index": "3",
    "parent_close_time": 666569690,
    "parent_hash": "F2E0CBB2E90DF9CA1DFB35451A0B19141BE5EE62796408B0689F02E2DC4301AF",
    "seqNum": "3",
    "totalCoins": "100000000000000000",
    "total_coins": "100000000000000000",
    "transaction_hash": "0000000000000000000000000000000000000000000000000000000000000000"
  },
  "startLedger": "B5DA94FCB914D11FC0C912C860EDAA6334CE511A299B965946A1C325BE0189E8"
}
```

# Example run
```
cargo run --release -- -i jlog2.txt -m -j as_json.json -g groups_report.txt -h histogram_report.txt -l latency_report.txt |& tee errors.txt
```
