# GetAllHostsQuery

Query parameters for filtering the hosts to get

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**limit** | **int** | Number of items to retrieve | 
**offset** | **int** | Position in the whole list to start retrieving from | 
**global_filter** | **str** | An optional general filter to apply | [optional] 
**host_filter** | **str** | An optional host specific filter to apply | [optional] 

## Example

```python
from kraken_sdk.models.get_all_hosts_query import GetAllHostsQuery

# TODO update the JSON string below
json = "{}"
# create an instance of GetAllHostsQuery from a JSON string
get_all_hosts_query_instance = GetAllHostsQuery.from_json(json)
# print the JSON string representation of the object
print GetAllHostsQuery.to_json()

# convert the object into a dict
get_all_hosts_query_dict = get_all_hosts_query_instance.to_dict()
# create an instance of GetAllHostsQuery from a dict
get_all_hosts_query_form_dict = get_all_hosts_query.from_dict(get_all_hosts_query_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


