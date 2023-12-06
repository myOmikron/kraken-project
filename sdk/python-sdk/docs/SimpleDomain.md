# SimpleDomain

A simple representation of a domain in a workspace

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** |  | 
**domain** | **str** |  | 
**comment** | **str** |  | 
**workspace** | **str** |  | 
**created_at** | **datetime** |  | 

## Example

```python
from kraken_sdk.models.simple_domain import SimpleDomain

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleDomain from a JSON string
simple_domain_instance = SimpleDomain.from_json(json)
# print the JSON string representation of the object
print SimpleDomain.to_json()

# convert the object into a dict
simple_domain_dict = simple_domain_instance.to_dict()
# create an instance of SimpleDomain from a dict
simple_domain_form_dict = simple_domain.from_dict(simple_domain_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


