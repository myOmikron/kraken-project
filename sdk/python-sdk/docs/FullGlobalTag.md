# FullGlobalTag

The full representation of a full

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** |  | 
**name** | **str** |  | 
**color** | [**Color**](Color.md) |  | 

## Example

```python
from kraken_sdk.models.full_global_tag import FullGlobalTag

# TODO update the JSON string below
json = "{}"
# create an instance of FullGlobalTag from a JSON string
full_global_tag_instance = FullGlobalTag.from_json(json)
# print the JSON string representation of the object
print FullGlobalTag.to_json()

# convert the object into a dict
full_global_tag_dict = full_global_tag_instance.to_dict()
# create an instance of FullGlobalTag from a dict
full_global_tag_form_dict = full_global_tag.from_dict(full_global_tag_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


