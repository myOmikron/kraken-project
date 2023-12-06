# WorkspaceInvitationList

A list of invitations to workspaces

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**invitations** | [**List[FullWorkspaceInvitation]**](FullWorkspaceInvitation.md) | All invitations of the current user | 

## Example

```python
from kraken_sdk.models.workspace_invitation_list import WorkspaceInvitationList

# TODO update the JSON string below
json = "{}"
# create an instance of WorkspaceInvitationList from a JSON string
workspace_invitation_list_instance = WorkspaceInvitationList.from_json(json)
# print the JSON string representation of the object
print WorkspaceInvitationList.to_json()

# convert the object into a dict
workspace_invitation_list_dict = workspace_invitation_list_instance.to_dict()
# create an instance of WorkspaceInvitationList from a dict
workspace_invitation_list_form_dict = workspace_invitation_list.from_dict(workspace_invitation_list_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


