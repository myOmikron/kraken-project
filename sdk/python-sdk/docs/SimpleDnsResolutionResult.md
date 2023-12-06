# SimpleDnsResolutionResult

A simple representation of a dns resolution result

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The primary key | 
**attack** | **str** | The attack which produced this result | 
**created_at** | **datetime** | The point in time, this result was produced | 
**source** | **str** | The source address | 
**destination** | **str** | The destination address | 
**dns_record_type** | **str** | The type of DNS Record | 

## Example

```python
from kraken_sdk.models.simple_dns_resolution_result import SimpleDnsResolutionResult

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleDnsResolutionResult from a JSON string
simple_dns_resolution_result_instance = SimpleDnsResolutionResult.from_json(json)
# print the JSON string representation of the object
print SimpleDnsResolutionResult.to_json()

# convert the object into a dict
simple_dns_resolution_result_dict = simple_dns_resolution_result_instance.to_dict()
# create an instance of SimpleDnsResolutionResult from a dict
simple_dns_resolution_result_form_dict = simple_dns_resolution_result.from_dict(simple_dns_resolution_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


