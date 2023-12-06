# SimpleHost

The simple representation of a host

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The primary key of the host | 
**ip_addr** | **str** | The ip address of the host | 
**os_type** | [**OsType**](OsType.md) |  | 
**comment** | **str** | A comment | 
**workspace** | **str** | The workspace this host is in | 
**created_at** | **datetime** | The point in time, the record was created | 

## Example

```python
from kraken_sdk.models.simple_host import SimpleHost

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleHost from a JSON string
simple_host_instance = SimpleHost.from_json(json)
# print the JSON string representation of the object
print SimpleHost.to_json()

# convert the object into a dict
simple_host_dict = simple_host_instance.to_dict()
# create an instance of SimpleHost from a dict
simple_host_form_dict = simple_host.from_dict(simple_host_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


