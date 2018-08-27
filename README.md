# Lustre Collector

[![Build Status](https://travis-ci.com/whamcloud/lustre-collector.svg?branch=master)](https://travis-ci.com/whamcloud/lustre-collector)

This repo provides data fetching and extraction capabilities for Lustre statisics.

It is provided as a standalone binary that can be called to retrieve stats in the desired output.

## Usage

```bash
# Will return OST related stats in JSON format
lustre_collector --host=oss1.domain --target_type=oss

# Will return OST related stats in YAML format
lustre_collector --host=oss1.domain --target_type=oss --format=yaml

# Will return MGS related stats in YAML format
lustre_collector --host=oss1.domain --target_type=mgs
```
