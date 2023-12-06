# SimpleHostAliveResult

A simple representation of a host alive result

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The primary key | 
**attack** | **str** | The attack which produced this result | 
**created_at** | **datetime** | The point in time, this result was produced | 
**host** | **str** | A host that responded | 

## Example

```python
from kraken_sdk.models.simple_host_alive_result import SimpleHostAliveResult

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleHostAliveResult from a JSON string
simple_host_alive_result_instance = SimpleHostAliveResult.from_json(json)
# print the JSON string representation of the object
print SimpleHostAliveResult.to_json()

# convert the object into a dict
simple_host_alive_result_dict = simple_host_alive_result_instance.to_dict()
# create an instance of SimpleHostAliveResult from a dict
simple_host_alive_result_form_dict = simple_host_alive_result.from_dict(simple_host_alive_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


