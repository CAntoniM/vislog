# VISLOG

A simple app for working with Visibroker's default logging format

## Usage

vislog reads files and by default simple reformats the message into the following
format

```
<LOG_LEVEL>: <Message>
```

so

```bash
vislog example.log
```

will output

```log
DEBUG: this is an example
```

This is done to allow you to de noise the quite busy visibroker logs into a more
readable format.

### Changing output format

vislog allows you to modify the output format of the messages uses `-f` flag
to do this simply write the string fromat you wish your message to following 
using wrapping the variable names in `{}` the list of avalible varibles are
listed below 

| Name      | Description                                            |
| --------- | ------------------------------------------------------ |
| pid       | the process id of the application                      |
| time      | the time of the log message                            |
| logger    | the name of the logger to print the message            |
| component | the name of the component the message originated in    |
| file      | the name of the file that message came from            |
| line      | the line number where the file came from               |
| level     | the level of the log message see visibroker log levels |
| message   | the message that was sent                              |

so you can format the message as follows:

```bash
vislog -f "{time} : {file}:{line} [{level}] -> {message} " example.log
```

and get the following output

```log
2024-07-09 09:10:07.000612542 : CSIV2IORInterceptor.cpp:231 [DEBUG] -> *** Server Interceptor installed for POA: "/exampleSERVER"
```

### Pattern matching the message

vislog allow you to filter the messages it returns by matching on a regular
expression this is done via the `-m` for example you can search for the word "test"
in a log file with the following command

```bash
vislog -m "test" example.log
```


### matching on process id

vislog allows you to filter on the process id of the process that output the log
message using the `-p` flag

```bash
vislog -p 22222 example.log
```

### matching on thread id

vislog allows you to filter a log based on the thread id of the thread that
spawned the message using the -t flag

```bash
vislog -t 1234 example.log
```

### matching on source file that generated the message

to filter the logs that are printed by vislog based on the source file that
printed the message using the `-s` flag

```bash
vislog -s vdelegate.cpp example.log
```

### matching on log level

to filter to logs based on the level by witch they where emited you can use the
`-l` flag for the valid values please refer to the visibroker for cpp developers
guide

```bash
vislog -l ERROR example.log
```

### maching on the component that generated the message

visibroker is comprised of serveral components you can get the logs from a
particular component using the `-c` flag this flag allow you match on the compnent name

```bash
vislog -c server example.log
```

### filtering logs based on time

vis log allows for two types of filtering based on the time a log message
happened on these are --before (-b) and --after (-a) which define you want to see
all log messages that happened before or after a particular time.

both of these filters can be applied at the same time to give the logs that
happened in a given time span

the format of the times used for both flags follows the same format as the
visibroker log message

e.g.

```bash
vislog --before "Tue Jul  9 09:09:39 2024 000000us" example.log
```

or 

```bash
vislog --after "Tue Jul  9 09:09:39 2024 000000us" example.log
```

to change the format of the date given in both the before and after flags use
using the `--date_fmt` this uses the formatting found [here](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)

### filtering the logs based on logger name

using the --logger flag you can filter the visibroker logs based on the name of
of the logger the that generated the message 

```bash
vislog --logger default example.log
```

## Building 

The app is built using the rust language as such you will be required to install
the rust toolchain see [the rust install guide](https://www.rust-lang.org/tools/install)
and then use the following commands to build the app

```bash
cargo build --release
```

this will build the app and place it in target/release/vislog

if you want to install the app onto your systems path use the following command

```bash
cargo install --path .
```

## Development

for future development we need to do the following 

1. Add reading from std::in
    currently we just read from the files provided on the command line. this is
    fine however is prevents usage when debugging the applicaiton as we can not
    filter or reformat the messages as the are created rather than having to
    only do it after.
1. Make the handling of errors better
    currently all error basically just get passed upto a panic state this is not
    very useful for anyone other than me who wants to use this app
2. Make the paser more efficent 
    currently the parser does a whole bunch of undeed reading and copying to 
    make the development of the app faster. e.g. we create and delete copys of
    strings all over the place to make handling them easier


