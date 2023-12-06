# FullWorkspaceTag

The full representation of a full workspace tag

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** |  | 
**name** | **str** |  | 
**color** | [**Color**](Color.md) |  | 
**workspace** | **str** |  | 

## Example

```python
from kraken_sdk.models.full_workspace_tag import FullWorkspaceTag

# TODO update the JSON string below
json = "{}"
# create an instance of FullWorkspaceTag from a JSON string
full_workspace_tag_instance = FullWorkspaceTag.from_json(json)
# print the JSON string representation of the object
print FullWorkspaceTag.to_json()

# convert the object into a dict
full_workspace_tag_dict = full_workspace_tag_instance.to_dict()
# create an instance of FullWorkspaceTag from a dict
full_workspace_tag_form_dict = full_workspace_tag.from_dict(full_workspace_tag_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


