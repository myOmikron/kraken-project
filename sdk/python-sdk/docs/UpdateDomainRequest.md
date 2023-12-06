# UpdateDomainRequest

The request to update a domain

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**comment** | **str** |  | [optional] 
**global_tags** | **List[str]** |  | [optional] 
**workspace_tags** | **List[str]** |  | [optional] 

## Example

```python
from kraken_sdk.models.update_domain_request import UpdateDomainRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateDomainRequest from a JSON string
update_domain_request_instance = UpdateDomainRequest.from_json(json)
# print the JSON string representation of the object
print UpdateDomainRequest.to_json()

# convert the object into a dict
update_domain_request_dict = update_domain_request_instance.to_dict()
# create an instance of UpdateDomainRequest from a dict
update_domain_request_form_dict = update_domain_request.from_dict(update_domain_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


