# kraken_sdk.AdminWorkspacesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_all_workspaces_admin**](AdminWorkspacesApi.md#get_all_workspaces_admin) | **GET** /api/v1/admin/workspaces | Retrieve all workspaces
[**get_workspace_admin**](AdminWorkspacesApi.md#get_workspace_admin) | **GET** /api/v1/admin/workspaces/{uuid} | Retrieve a workspace by id


# **get_all_workspaces_admin**
> GetAllWorkspacesResponse get_all_workspaces_admin()

Retrieve all workspaces

Retrieve all workspaces

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.get_all_workspaces_response import GetAllWorkspacesResponse
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
    api_instance = kraken_sdk.AdminWorkspacesApi(api_client)

    try:
        # Retrieve all workspaces
        api_response = await api_instance.get_all_workspaces_admin()
        print("The response of AdminWorkspacesApi->get_all_workspaces_admin:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AdminWorkspacesApi->get_all_workspaces_admin: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

[**GetAllWorkspacesResponse**](GetAllWorkspacesResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns all workspaces |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_workspace_admin**
> FullWorkspace get_workspace_admin(uuid)

Retrieve a workspace by id

Retrieve a workspace by id

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.full_workspace import FullWorkspace
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
    api_instance = kraken_sdk.AdminWorkspacesApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Retrieve a workspace by id
        api_response = await api_instance.get_workspace_admin(uuid)
        print("The response of AdminWorkspacesApi->get_workspace_admin:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AdminWorkspacesApi->get_workspace_admin: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

[**FullWorkspace**](FullWorkspace.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns the workspace with the given id |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

