# ACE Champion
ACE Champion is a dedicated server manager for Assetto Corsa EVO.

## Usage
```
ace-champ -S /path/to/server/install launch cfg/
```
where `cfg/` is the a directory containing settings.json (`-configjson`) and season.json (`-seasonjson`)
A pid and log will also be created in this directory.

### Examples
```
# launch server
ace-champ -S /data/steam/AC-EVO-Server/ launch pcup-test/

# kill server (signal passed to AssettoCorsaEVOServer.exe)
kill `cat pcup-test/ace-champ.pid`
```

## Installation
```
make
make install
```
this will install to `~/.local/bin`.

To install other locations override PREFIX
```
make install PREFIX=/path/to/install
```


## Features
* launch via wine and fork to background
* redirect output to logfile
* pass SIGINT/SIGTERM to process

## Roadmap
* ServerLauncher.exe command conversion
* Track Rotation
* Championship Standings
