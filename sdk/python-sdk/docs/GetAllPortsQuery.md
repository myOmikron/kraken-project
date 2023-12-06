# GetAllPortsQuery

Query parameters for filtering the ports to get

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**limit** | **int** | Number of items to retrieve | 
**offset** | **int** | Position in the whole list to start retrieving from | 
**host** | **str** | Only get ports associated with a specific host | [optional] 
**global_filter** | **str** | An optional general filter to apply | [optional] 
**port_filter** | **str** | An optional port specific filter to apply | [optional] 

## Example

```python
from kraken_sdk.models.get_all_ports_query import GetAllPortsQuery

# TODO update the JSON string below
json = "{}"
# create an instance of GetAllPortsQuery from a JSON string
get_all_ports_query_instance = GetAllPortsQuery.from_json(json)
# print the JSON string representation of the object
print GetAllPortsQuery.to_json()

# convert the object into a dict
get_all_ports_query_dict = get_all_ports_query_instance.to_dict()
# create an instance of GetAllPortsQuery from a dict
get_all_ports_query_form_dict = get_all_ports_query.from_dict(get_all_ports_query_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


