# SimpleLeech

The simple representation of a leech

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** |  | 
**name** | **str** |  | 
**address** | **str** |  | 

## Example

```python
from kraken_sdk.models.simple_leech import SimpleLeech

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleLeech from a JSON string
simple_leech_instance = SimpleLeech.from_json(json)
# print the JSON string representation of the object
print SimpleLeech.to_json()

# convert the object into a dict
simple_leech_dict = simple_leech_instance.to_dict()
# create an instance of SimpleLeech from a dict
simple_leech_form_dict = simple_leech.from_dict(simple_leech_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


