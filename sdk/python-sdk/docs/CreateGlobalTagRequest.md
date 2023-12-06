# CreateGlobalTagRequest

The request to create a global tag

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** | Name of the tag | 
**color** | [**Color**](Color.md) |  | 

## Example

```python
from kraken_sdk.models.create_global_tag_request import CreateGlobalTagRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateGlobalTagRequest from a JSON string
create_global_tag_request_instance = CreateGlobalTagRequest.from_json(json)
# print the JSON string representation of the object
print CreateGlobalTagRequest.to_json()

# convert the object into a dict
create_global_tag_request_dict = create_global_tag_request_instance.to_dict()
# create an instance of CreateGlobalTagRequest from a dict
create_global_tag_request_form_dict = create_global_tag_request.from_dict(create_global_tag_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


