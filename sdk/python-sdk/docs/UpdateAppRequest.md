# UpdateAppRequest


## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** |  | [optional] 
**redirect_uri** | **str** |  | [optional] 

## Example

```python
from kraken_sdk.models.update_app_request import UpdateAppRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateAppRequest from a JSON string
update_app_request_instance = UpdateAppRequest.from_json(json)
# print the JSON string representation of the object
print UpdateAppRequest.to_json()

# convert the object into a dict
update_app_request_dict = update_app_request_instance.to_dict()
# create an instance of UpdateAppRequest from a dict
update_app_request_form_dict = update_app_request.from_dict(update_app_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


