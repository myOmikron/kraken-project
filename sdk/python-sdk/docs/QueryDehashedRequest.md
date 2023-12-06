# QueryDehashedRequest

The request to query the dehashed API

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**query** | [**Query**](Query.md) |  | 
**workspace_uuid** | **str** | The workspace to execute the attack in | 

## Example

```python
from kraken_sdk.models.query_dehashed_request import QueryDehashedRequest

# TODO update the JSON string below
json = "{}"
# create an instance of QueryDehashedRequest from a JSON string
query_dehashed_request_instance = QueryDehashedRequest.from_json(json)
# print the JSON string representation of the object
print QueryDehashedRequest.to_json()

# convert the object into a dict
query_dehashed_request_dict = query_dehashed_request_instance.to_dict()
# create an instance of QueryDehashedRequest from a dict
query_dehashed_request_form_dict = query_dehashed_request.from_dict(query_dehashed_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


