# dop

Process, transform and query JSON/YAML/TOML, from the shell.

Like awk, where scripts can be used to manipulate each line of a given text
stream, *dop* works similarly: instead of text lines, it traverses across
fields/values from a hierarchical data format such as JSON/YAML/TOML.

```sh
$ echo '{"list":[1,2,3]}' | dop -e '

-- Below is Lua code

if type(VALUE) == "number" then
    set(VALUE * 2)
end
'

# Prints out:
# {"list":[2,4,6]}
```

*dop* stands for data operations.

## Examples

### Modifying data 

```
$ echo '[1,2,3]' | dop -e 'set(VALUE * 2)'

# Prints out:
# [2,4,6]
```

### Adding new fields

```
echo '{"my_list":[1,2,3]}' | dop -E '
set("my_list[]", 4)
set("foo", "bar")
'

# Prints out:
# {"foo":"bar","my_list":[1,2,3,4]}
```

### Handling YAML

```
echo '
list:
- 1
- 2
- 3
' | dop -i yaml -e 'if type(VALUE) == "number" then set(VALUE * 2) end'

# Prints out:
# list:
# - 2
# - 4
# - 6
```


> Note above we passed `-i yaml` to tell *dop* the input format is YAML. But in
> fact it would work even without that parameter as it's is able to identify
> the input format automatically. It just makes execution a bit faster as the
> program will not try to guess the data format on its own.

### Removing fields

```
$ echo '{"some_list":[1,2,3]}' | dop -e '
if KEY == "some_list[2]" then
    unset()
end
'

# Prints out:
# {"some_list":[1,2]}
```

### Querying data

```
$ echo '{"data":{"some_list":[1,2,3]}}' | dop -q data.some_list

# Prints out:
# [1,2,3]
```

### As just a format convertor

With the `-o` option you can set which output format you'd like:

```
$ echo '{"data":{"some_list":[1,2,3]}}' | dop -o yaml

# Prints out:
# data:
#  some_list:
#  - 1
#  - 2
#  - 3

$ echo '{"data":{"some_list":[1,2,3]}}' | dop -o toml

# Prints out:
# [data]
# some_list = [1, 2, 3]
```

### Execute script once (no traversal)

```sh
echo '{"data":{"some_list":[1,2,3]}}' | \
dop -E '
set("data.some_list[0]", 10)
set("data.some_list[1]", 20)
set("data.some_list[2]", 30)
'

# Prints out:
# {"data":{"some_list":[10, 20, 30]}}  
```

> Note above we used `-E` (alias `--execute-once`) instead of just `-e`

### Execute shell commands and capture output

```
$ echo '
{
  "domains": {
    "google.com": "???",
    "microsoft.com": "???"
  }
}
' | dop -k 'domains\.[a-z]' -e '
set(exec(
    string.format(
        "dig +short %s | head -n 1",
        KEY
    )
).output)
'

# Prints out:
# {"domains":{"google.com":"142.251.39.238","microsoft.com":"20.236.44.162"}}
```

### Overwriting output

```
echo '[1,2,3]' | dop \
  --begin 'result = {counter = 0}' \
  -e 'result.counter = result.counter + 1' \
  -p result

# Prints out:
# {"counter": 3}
```

## Options Reference

```
-e <SCRIPT>, --execute <SCRIPT>
       Traverses the whole data structure (unless you're filtering with -k),
       visiting every field and executing <SCRIPT> (a stringified shell script)
       each time. The shell script is able to perform operations on the
       traversed data, for example:

       $ dop -e 'set("new value")' < ...

       In the example above we pass stringified Lua script in "-e".
       Alternatively a script file can be passed:

       $ dop -e some_script.lua < ...

-E <SCRIPT>, --execute-once <SCRIPT>
       Like --execute however no traversal happens therefore execution happens
       only once.

       $ dop -E '
       set("foo", "bar")
       set("data.some_list[5]", 123)
       ' < ...

-b <SCRIPT>, --begin <SCRIPT>
       Executes a given Lua script before data processing begins. At this point
       you can define variables that will be carried over across execution
       during data processing. This option is analogous with AWK's "BEGIN{}".

       $ printf '[1,2,3]' | dop -b 'sum = 0' -e 'sum = sum + VALUE' --print-var sum
       # Prints out:
       # 6

-k <VALUE>, --key-filter <VALUE>
       Search for keys based on given regular expression. The script will only
       be executed for fields which key match the search.

       If not used, dop simply executes your shell script for all values,
       traversing your whole data structure (unless --execute-once is used.)

       NOTE: This option does not affect the output at all. It filters only
       which values will be passed to your script.

-K <VALUE>, --key-equal <VALUE>
       Like -k except this is for exact key matching, not regular expression.

-i <FORMAT>, --input-format <FORMAT>
       If not set, dop will try to detect in which data format your input is
       formatted, therefore telling dop the format is optional. However
       execution will be faster if you use this option.

-o <FORMAT>, --output-format <FORMAT>
       If not set, dop will output in the same format as your input. If you'd
       like the output format to be different from the input format, use this
       option.

-q <VALUE>, --query <VALUE>
       By default dop prints out the whole data structure (except the fields
       which were removed.) Use the query option if you'd like to print only a
       certain field from your data structure.

       NOTE: You can use dop to query data even if you're not modifying
       anything through a script:

       $ echo '{"list":[1,2,3]}' | dop -q 'list[1]'
       # Prints out:
       # 2

-P, --pretty
       By default dop prints in compact style. Use this option for a nice
       output.

       NOTE: Only JSON supports pretty print. All other formats will just
       ignore this option.

-v, --verbose
       Print out useful debugging information to STDERR, while your data is
       processed. You may also use log("some text") to print out debugging
       messages from your script, which will only be visible if -v is used.
```

## Script library reference

Scripts are equipped with a set of utility functions for operating on input
data. The complete list of available functions is provided below.

```
set(<VALUE>)
       Modifies the current value during traversal.

       $ echo '{"foo":"bar"}' | dop -e 'set("new value!")'
       # Prints out:
       # {"foo":"new value!"}

       "set" with only <VALUE> can only be used with traversal execution (aka
       -e instead of -E).

       OPTIONS:
       -s,--string: Forces <VALUE> to be set as a string. If -s is not used,
       dop will parse the value automatically, for example "10" will be set as
       a number; "true" will be set as a boolean, etc. But when -s is used, the
       value is forced to be a string without any parsing attempt.

set(<KEY>, <VALUE>, [<OPTIONS>])
       Modifies <KEY> attributing <VALUE> to it.
       Assigns <VALUE> to <KEY>.

       $ echo '{"foo":"bar"}' | dop -E 'set("hello", "world")'
       # Prints out:
       # {"foo":"bar","hello":"world"}

       OPTIONS:
       - force: create parent objects if/as needed:

         $ echo '{"foo":"bar"}' | dop -E '
            set("one.two.three.four", "five", {force = true})
           '
         # Prints out:
         # {"foo":"bar","one":{"two":{"three":{"four":"five"}}}}

       NOTES:
       Setting with key/value combination is supported in both
       traversal-execution (-e) and execute-once (-E) methods.

get
       Gets the current value during traversal.

       $ echo '[1,2,3]' | dop -e 'set(get() * 2)'
       # Prints out:
       # [2,4,6]

       NOTE: get() (with no arguments) has the same result as just reading
       global variable "VALUE".

       NOTE: During execution-once (aka "-E"), get() with no arguments will
       retrieve the whole data input, whereas during traversal mode (aka -e)
       the currently traversed value is retrieved.

unset
       Deletes the current value while in traversal.

       echo '{"list":[1,2,3]}' | dop -e '
       if type(VALUE) == "number" then
           unset()
       end
       '
       # Prints out:
       # {"list":[]}

unset <KEY>
       Deletes <KEY>.

       echo '{"list":[1,2,3]}' | dop -E 'unset("list[1]")'
       # Prints out:
       # {"list":[1,3]}

get <KEY>
       Gets any value from the input data.

       $ echo '{"base":50,"list":[1,2,3]}' | dop -k 'list.\d' -e '
       set(get("base") * get())
       '
       # Prints out:
       # {"base":50,"list":[50,100,150]}

exec <COMMAND>
       Executes a shell command. An object is returned containing "output"
       (string) and "status" (number).

       $ echo '[1,2,3]' | dop -e '
       set(exec(string.format("echo $((%d * 2)) | bc", VALUE)).output)
       '
       # Prints out:
       # ["2","4","6"]
```

## FAQ

### Do I need the Lua runtime installed on my machine?

No. You don't need to install it, as the lua interpreter/runtime is built into
*dop*'s executable.

### Why not just use jq or similar tools?

jq is good but I often needed to search their manuals to remember their
scripting syntax, even for trivial data modification tasks. *dop* takes a
different approach:
- Use a general-purpose scripting language
- Write transformations in a familiar programming style
- Reduce mental overhead for complex logic

### Inspired by

*dop* is mainly inspired by *awk*:
- awk → script runs once per text line
- dop → script runs once per structured data field

## License

[MIT](LICENSE)
