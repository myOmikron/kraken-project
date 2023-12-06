# UpdateGlobalTag

The request to update a global tag

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** |  | [optional] 
**color** | [**Color**](Color.md) |  | [optional] 

## Example

```python
from kraken_sdk.models.update_global_tag import UpdateGlobalTag

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateGlobalTag from a JSON string
update_global_tag_instance = UpdateGlobalTag.from_json(json)
# print the JSON string representation of the object
print UpdateGlobalTag.to_json()

# convert the object into a dict
update_global_tag_dict = update_global_tag_instance.to_dict()
# create an instance of UpdateGlobalTag from a dict
update_global_tag_form_dict = update_global_tag.from_dict(update_global_tag_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


