# SimpleBruteforceSubdomainsResult

A simple representation of a bruteforce subdomains result

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
from kraken_sdk.models.simple_bruteforce_subdomains_result import SimpleBruteforceSubdomainsResult

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleBruteforceSubdomainsResult from a JSON string
simple_bruteforce_subdomains_result_instance = SimpleBruteforceSubdomainsResult.from_json(json)
# print the JSON string representation of the object
print SimpleBruteforceSubdomainsResult.to_json()

# convert the object into a dict
simple_bruteforce_subdomains_result_dict = simple_bruteforce_subdomains_result_instance.to_dict()
# create an instance of SimpleBruteforceSubdomainsResult from a dict
simple_bruteforce_subdomains_result_form_dict = simple_bruteforce_subdomains_result.from_dict(simple_bruteforce_subdomains_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


