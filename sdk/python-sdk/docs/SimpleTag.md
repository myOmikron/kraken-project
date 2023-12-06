# SimpleTag

A simple tag

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** |  | 
**name** | **str** |  | 
**color** | [**Color**](Color.md) |  | 
**tag_type** | [**TagType**](TagType.md) |  | 

## Example

```python
from kraken_sdk.models.simple_tag import SimpleTag

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleTag from a JSON string
simple_tag_instance = SimpleTag.from_json(json)
# print the JSON string representation of the object
print SimpleTag.to_json()

# convert the object into a dict
simple_tag_dict = simple_tag_instance.to_dict()
# create an instance of SimpleTag from a dict
simple_tag_form_dict = simple_tag.from_dict(simple_tag_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


