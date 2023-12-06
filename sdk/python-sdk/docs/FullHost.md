# FullHost

The full representation of a host

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The primary key of the host | 
**ip_addr** | **str** | The ip address of the host | 
**os_type** | [**OsType**](OsType.md) |  | 
**comment** | **str** | A comment | 
**workspace** | **str** | The workspace this host is in | 
**tags** | [**List[SimpleTag]**](SimpleTag.md) | The list of tags this host has attached to | 
**sources** | [**SimpleAggregationSource**](SimpleAggregationSource.md) |  | 
**created_at** | **datetime** | The point in time, the record was created | 

## Example

```python
from kraken_sdk.models.full_host import FullHost

# TODO update the JSON string below
json = "{}"
# create an instance of FullHost from a JSON string
full_host_instance = FullHost.from_json(json)
# print the JSON string representation of the object
print FullHost.to_json()

# convert the object into a dict
full_host_dict = full_host_instance.to_dict()
# create an instance of FullHost from a dict
full_host_form_dict = full_host.from_dict(full_host_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


