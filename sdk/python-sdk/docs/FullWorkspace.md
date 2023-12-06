# FullWorkspace

A full version of a workspace

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** |  | 
**name** | **str** |  | 
**description** | **str** |  | [optional] 
**owner** | [**SimpleUser**](SimpleUser.md) |  | 
**attacks** | [**List[SimpleAttack]**](SimpleAttack.md) |  | 
**members** | [**List[SimpleUser]**](SimpleUser.md) |  | 
**created_at** | **datetime** |  | 

## Example

```python
from kraken_sdk.models.full_workspace import FullWorkspace

# TODO update the JSON string below
json = "{}"
# create an instance of FullWorkspace from a JSON string
full_workspace_instance = FullWorkspace.from_json(json)
# print the JSON string representation of the object
print FullWorkspace.to_json()

# convert the object into a dict
full_workspace_dict = full_workspace_instance.to_dict()
# create an instance of FullWorkspace from a dict
full_workspace_form_dict = full_workspace.from_dict(full_workspace_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


