# Edit Count

[![Build Status](https://travis-ci.com/eloc3147/edit_count.svg?branch=master)](https://travis-ci.com/eloc3147/edit_count)

Blurb

## Configuration

Edit Count is configured via a configuration file in the [TOML](https://github.com/toml-lang/toml) format.

|      OS | Config Path                                                  |
| ------- | ------------------------------------------------------------ |
| Windows | `%APPDATA%\edit_count\edit_count\settings.toml`        |
| MacOS   | `$HOME/Library/Application Support/edit_count/settings.toml` |
| *NIX    | `$HOME/.config/edit_count/settings.toml`                     |

The config file has the following format.

```TOML
# Filesystem event de-duplication buffer, in milliseconds
# Updates to folders will not be processes until the specified delay has passed
# to allow time for duplicate events to be removed.
# Decreasing this value will increase responsiveness, at the cost of CPU usage
watch_frequency=1000

# See Directory Layout section below
[directory_layout]
    raw_dirs = ['']
    render_dirs = ['']
```

## Directory Layout

I intend for the directory layout system to be highly customizable.
I have only implemented a few path operators so far, but if you are unable to configure Edit Count for your workflow, please submit an issue and I'll see what I can do.

### Configuring Directory Layout

Directory layout is configured with the `directory_layout` table as shown below.

```TOML
[directory_layout]
    raw_dirs = ['']
    render_dirs = ['']
```

The directory layout table has two keys. `raw_dirs` and `render_dirs`.
Both of these keys should be arrays of strings, formatted using the Directory Path syntax.

### Directory Path Syntax

Directory Paths are strings containing a path, an optional group operator, and an album operator.

Operators are how the Layout Parser knows how to interperet directories.
Operators are converted into directories and therefore **must** be surrounded by path seperators on both sides.
If a group operator is used, it **must** come before the album operator.

Operators and arguments **must** be placed in square brackets.

#### Available Operators

**OPERATOR** ( optional argument ) *Operator name*

**G** ( depth ) *Group operator* :
> The group operator marks a path segment as the group.  
> The group operator takes an optional `depth` argument.
> The `depth` argument configures how many levels of subdirectories should be considered the group name.
> `depth` must be a positive integer, and defaults to 1 if no value is provided.

**A** ( depth ) *Album Operator* :
> The album operator marks a path segment as the album.  
> The group operator takes an optional `depth` argument.
> The `depth` argument configures how many levels of subdirectories should be considered the album name.
> `depth` can either be a positive integer, a range, or empty, defaulting to 1.
> Ranges are specified as `Min.Max` where Min and Max are positive integers.
> Min and Max are optional, with Min defaulting to 1, and Max defaulting to Infinite.

#### Examples

Consider the following folder structure:

```
Photos/
  2007/
    Mexico Trip/
      1.nef
      2.nef
      Fishing/
        fish.nef
    Halloween/
      1.dng
      2.dng
      3.dng
  2008/
    Birthday/
      panorama.psd
  2009/
```

The following examples will be performed on the above.

`Photos/2007/[A]/` :
> ```
> albums: [
>   Mexico Trip,
>   Halloween
> ]
> ```

`Photos/[A.]/` :
> ```
> albums: [
>   2007/Mexico Trip,
>   2007/Mexico Trip/Fishing,
>   2007/Halloween,
>   2008/Birthday
> ]
> ```

`Photos/[G]/[A]/` :
> ```
> groups:  [
>   {
>     name: 2007,
>     albums: [
>       Mexico Trip,
>       Halloween
>     ]
>   },
>   {
>     name: 2008,
>     albums: [
>       Birthday
>     ]
>   }
> ]
> ```
