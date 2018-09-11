# Lustre Collector

[![Build Status](https://travis-ci.com/whamcloud/lustre-collector.svg?branch=master)](https://travis-ci.com/whamcloud/lustre-collector)

This repo provides a parsed representation of common Lustre statistics.

It is provided as a standalone binary that can be called to retrieve stats in the desired output (Currently either JSON | YAML).

## Usage

```bash
# Will return stats in JSON format
lustre_collector

# Will return stats in YAML format
lustre_collector --format=Yaml
```
