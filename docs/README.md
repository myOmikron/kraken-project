# Documentation of Kraken project

This is the documentation of [Kraken project](https://github.com/myOmikron/kraken-project). 
It uses `mkdocs` with the beautiful `material` theme.

## Changes

In order to make changes and build the documentation following requirements are necessary:

- python
- python-pip

as well as the python modules:

```bash
python -m pip install -r requirements.txt
```

To build the documentation run:
```bash
mkdocs build
```

To spin up a self-refreshing development server:
```bash
mkdocs serve -a bind_addr:port
```

