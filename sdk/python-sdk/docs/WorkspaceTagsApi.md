# kraken_sdk.WorkspaceTagsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_workspace_tag**](WorkspaceTagsApi.md#create_workspace_tag) | **POST** /api/v1/workspaces/{uuid}/tags | Create a workspace tag.
[**delete_workspace_tag**](WorkspaceTagsApi.md#delete_workspace_tag) | **DELETE** /api/v1/workspaces/{w_uuid}/tags/{t_uuid} | Delete a workspace tag
[**get_all_workspace_tags**](WorkspaceTagsApi.md#get_all_workspace_tags) | **GET** /api/v1/workspaces/{uuid}/tags | Retrieve all workspace tags
[**update_workspace_tag**](WorkspaceTagsApi.md#update_workspace_tag) | **PUT** /api/v1/workspaces/{w_uuid}/tags/{t_uuid} | Update a workspace tag


# **create_workspace_tag**
> UuidResponse create_workspace_tag(uuid, create_workspace_tag_request)

Create a workspace tag.

Create a workspace tag.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.create_workspace_tag_request import CreateWorkspaceTagRequest
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
    api_instance = kraken_sdk.WorkspaceTagsApi(api_client)
    uuid = 'uuid_example' # str | 
    create_workspace_tag_request = kraken_sdk.CreateWorkspaceTagRequest() # CreateWorkspaceTagRequest | 

    try:
        # Create a workspace tag.
        api_response = await api_instance.create_workspace_tag(uuid, create_workspace_tag_request)
        print("The response of WorkspaceTagsApi->create_workspace_tag:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling WorkspaceTagsApi->create_workspace_tag: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **create_workspace_tag_request** | [**CreateWorkspaceTagRequest**](CreateWorkspaceTagRequest.md)|  | 

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
**200** | Workspace tag was created |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_workspace_tag**
> delete_workspace_tag(w_uuid, t_uuid)

Delete a workspace tag

Delete a workspace tag  Requires privileges to access the workspace this tag belongs to.

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
    api_instance = kraken_sdk.WorkspaceTagsApi(api_client)
    w_uuid = 'w_uuid_example' # str | 
    t_uuid = 't_uuid_example' # str | 

    try:
        # Delete a workspace tag
        await api_instance.delete_workspace_tag(w_uuid, t_uuid)
    except Exception as e:
        print("Exception when calling WorkspaceTagsApi->delete_workspace_tag: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **w_uuid** | **str**|  | 
 **t_uuid** | **str**|  | 

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
**200** | Workspace tag was deleted |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_all_workspace_tags**
> GetWorkspaceTagsResponse get_all_workspace_tags(uuid)

Retrieve all workspace tags

Retrieve all workspace tags

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.get_workspace_tags_response import GetWorkspaceTagsResponse
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
    api_instance = kraken_sdk.WorkspaceTagsApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Retrieve all workspace tags
        api_response = await api_instance.get_all_workspace_tags(uuid)
        print("The response of WorkspaceTagsApi->get_all_workspace_tags:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling WorkspaceTagsApi->get_all_workspace_tags: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

[**GetWorkspaceTagsResponse**](GetWorkspaceTagsResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Retrieve all workspace tags |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_workspace_tag**
> update_workspace_tag(w_uuid, t_uuid, update_workspace_tag)

Update a workspace tag

Update a workspace tag  One of the options must be set  Requires privileges to access the workspace this tags belongs to.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.update_workspace_tag import UpdateWorkspaceTag
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
    api_instance = kraken_sdk.WorkspaceTagsApi(api_client)
    w_uuid = 'w_uuid_example' # str | 
    t_uuid = 't_uuid_example' # str | 
    update_workspace_tag = kraken_sdk.UpdateWorkspaceTag() # UpdateWorkspaceTag | 

    try:
        # Update a workspace tag
        await api_instance.update_workspace_tag(w_uuid, t_uuid, update_workspace_tag)
    except Exception as e:
        print("Exception when calling WorkspaceTagsApi->update_workspace_tag: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **w_uuid** | **str**|  | 
 **t_uuid** | **str**|  | 
 **update_workspace_tag** | [**UpdateWorkspaceTag**](UpdateWorkspaceTag.md)|  | 

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
**200** | Workspace tag was updated |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

