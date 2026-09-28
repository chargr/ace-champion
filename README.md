# ACE Champion
ACE Champion is a dedicated server manager for Assetto Corsa EVO.

## Usage
```
ace-champ start -S /path/to/server/install cfg/
```
where `cfg/` is the a directory containing settings.json (`-configjson`) and season.json (`-seasonjson`)
A pid and log will also be created in this directory.

### Examples
```
# launch server installed in /data/steam/AC-EVO-Server
ace-champ start -S /data/steam/AC-EVO-Server/ pcup-test/

# stop server
ace-champ stop pcup-test/

# restart server
ace-champ restart pcup-test/
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
