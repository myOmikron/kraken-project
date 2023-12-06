# kraken_sdk.GlobalTagsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_global_tag**](GlobalTagsApi.md#create_global_tag) | **POST** /api/v1/admin/globalTags | Create a global tag.
[**delete_global_tag**](GlobalTagsApi.md#delete_global_tag) | **DELETE** /api/v1/admin/globalTags/{uuid} | Delete a global tag
[**get_all_global_tags**](GlobalTagsApi.md#get_all_global_tags) | **GET** /api/v1/globalTags | Retrieve all global tags
[**update_global_tag**](GlobalTagsApi.md#update_global_tag) | **PUT** /api/v1/admin/globalTags/{uuid} | Update a global tag


# **create_global_tag**
> UuidResponse create_global_tag(create_global_tag_request)

Create a global tag.

Create a global tag.  This action requires admin privileges.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.create_global_tag_request import CreateGlobalTagRequest
from kraken_sdk.models.uuid_response import UuidResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.GlobalTagsApi(api_client)
    create_global_tag_request = kraken_sdk.CreateGlobalTagRequest() # CreateGlobalTagRequest | 

    try:
        # Create a global tag.
        api_response = await api_instance.create_global_tag(create_global_tag_request)
        print("The response of GlobalTagsApi->create_global_tag:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling GlobalTagsApi->create_global_tag: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **create_global_tag_request** | [**CreateGlobalTagRequest**](CreateGlobalTagRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Global tag was created |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_global_tag**
> delete_global_tag(uuid)

Delete a global tag

Delete a global tag  Requires admin privileges.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.GlobalTagsApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Delete a global tag
        await api_instance.delete_global_tag(uuid)
    except Exception as e:
        print("Exception when calling GlobalTagsApi->delete_global_tag: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Global tag was deleted |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_all_global_tags**
> GetGlobalTagsResponse get_all_global_tags()

Retrieve all global tags

Retrieve all global tags

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.get_global_tags_response import GetGlobalTagsResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.GlobalTagsApi(api_client)

    try:
        # Retrieve all global tags
        api_response = await api_instance.get_all_global_tags()
        print("The response of GlobalTagsApi->get_all_global_tags:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling GlobalTagsApi->get_all_global_tags: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

[**GetGlobalTagsResponse**](GetGlobalTagsResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Retrieve all global tags |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_global_tag**
> update_global_tag(uuid, update_global_tag)

Update a global tag

Update a global tag  One of the options must be set  Requires admin privileges.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.update_global_tag import UpdateGlobalTag
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.GlobalTagsApi(api_client)
    uuid = 'uuid_example' # str | 
    update_global_tag = kraken_sdk.UpdateGlobalTag() # UpdateGlobalTag | 

    try:
        # Update a global tag
        await api_instance.update_global_tag(uuid, update_global_tag)
    except Exception as e:
        print("Exception when calling GlobalTagsApi->update_global_tag: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **update_global_tag** | [**UpdateGlobalTag**](UpdateGlobalTag.md)|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Global tag was updated |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

