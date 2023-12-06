# GetAllServicesQuery

Query parameters for filtering the services to get

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**limit** | **int** | Number of items to retrieve | 
**offset** | **int** | Position in the whole list to start retrieving from | 
**host** | **str** | Only get services associated with a specific host | [optional] 
**global_filter** | **str** | An optional general filter to apply | [optional] 
**service_filter** | **str** | An optional service specific filter to apply | [optional] 

## Example

```python
from kraken_sdk.models.get_all_services_query import GetAllServicesQuery

# TODO update the JSON string below
json = "{}"
# create an instance of GetAllServicesQuery from a JSON string
get_all_services_query_instance = GetAllServicesQuery.from_json(json)
# print the JSON string representation of the object
print GetAllServicesQuery.to_json()

# convert the object into a dict
get_all_services_query_dict = get_all_services_query_instance.to_dict()
# create an instance of GetAllServicesQuery from a dict
get_all_services_query_form_dict = get_all_services_query.from_dict(get_all_services_query_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


