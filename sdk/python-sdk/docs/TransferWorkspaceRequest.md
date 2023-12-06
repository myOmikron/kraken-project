# TransferWorkspaceRequest

The request to transfer a workspace to another account

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**user** | **str** | The uuid of the user that should receive the workspace | 

## Example

```python
from kraken_sdk.models.transfer_workspace_request import TransferWorkspaceRequest

# TODO update the JSON string below
json = "{}"
# create an instance of TransferWorkspaceRequest from a JSON string
transfer_workspace_request_instance = TransferWorkspaceRequest.from_json(json)
# print the JSON string representation of the object
print TransferWorkspaceRequest.to_json()

# convert the object into a dict
transfer_workspace_request_dict = transfer_workspace_request_instance.to_dict()
# create an instance of TransferWorkspaceRequest from a dict
transfer_workspace_request_form_dict = transfer_workspace_request.from_dict(transfer_workspace_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


