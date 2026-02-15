import axios from 'axios'
import { useAuthStore } from '../stores/auth'

const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// 请求拦截器 - 添加 token
api.interceptors.request.use(
  (config) => {
    const token = useAuthStore.getState().token
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// 响应拦截器 - 处理错误
api.interceptors.response.use(
  (response) => {
    return response.data
  },
  (error) => {
    if (error.response?.status === 401) {
      useAuthStore.getState().logout()
      window.location.href = '/login'
    }
    return Promise.reject(error)
  }
)

export default api

// API 响应类型
export interface ApiResponse<T> {
  code: number
  data: T
  message: string
}

// 认证 API
export const authApi = {
  login: (username: string, password: string) =>
    api.post<ApiResponse<{ token: string; user: any }>>('/auth/login', { username, password }),
  me: () => api.get<ApiResponse<any>>('/auth/me'),
  logout: () => api.post('/auth/logout'),
}

// 系统 API
export const systemApi = {
  getInfo: () => api.get<ApiResponse<any>>('/system/info'),
  getStats: () => api.get<ApiResponse<any>>('/system/stats'),
  getProcesses: () => api.get<ApiResponse<any[]>>('/system/processes'),
  getNetwork: () => api.get<ApiResponse<any[]>>('/system/network'),
  getServices: () => api.get<ApiResponse<any[]>>('/system/services'),
}

// 文件 API
export const fileApi = {
  list: (path: string = '/') =>
    api.get<ApiResponse<any[]>>('/files', { params: { path } }),
  getContent: (path: string) =>
    api.get<ApiResponse<string>>('/files/content', { params: { path } }),
  create: (path: string, type: 'file' | 'directory') =>
    api.post('/files', { path, type }),
  update: (path: string, content: string) =>
    api.put('/files', { path, content }),
  delete: (path: string) =>
    api.delete('/files', { params: { path } }),
}

// Docker API
export const dockerApi = {
  listContainers: () => api.get<ApiResponse<any[]>>('/containers'),
  startContainer: (id: string) => api.post(`/containers/${id}/start`),
  stopContainer: (id: string) => api.post(`/containers/${id}/stop`),
  restartContainer: (id: string) => api.post(`/containers/${id}/restart`),
  removeContainer: (id: string) => api.delete(`/containers/${id}`),
  containerLogs: (id: string) => api.get<ApiResponse<string[]>>(`/containers/${id}/logs`),
  listImages: () => api.get<ApiResponse<any[]>>('/images'),
  pullImage: (image: string) => api.post('/images/pull', { image }),
  removeImage: (id: string) => api.delete(`/images/${id}`),
}

// 网站管理 API
export const websiteApi = {
  list: () => api.get<ApiResponse<any[]>>('/websites'),
  create: (data: any) => api.post('/websites', data),
  update: (id: number, data: any) => api.put(`/websites/${id}`, data),
  delete: (id: number) => api.delete(`/websites/${id}`),
  configureSsl: (id: number, data: any) => api.post(`/websites/${id}/ssl`, data),
  reloadNginx: (id: number) => api.post(`/websites/${id}/reload`),
}

// AI 应用 API
export const appApi = {
  list: () => api.get<ApiResponse<any[]>>('/apps'),
  templates: () => api.get<ApiResponse<any[]>>('/apps/templates'),
  install: (data: any) => api.post('/apps/install', data),
  start: (id: number) => api.post(`/apps/${id}/start`),
  stop: (id: number) => api.post(`/apps/${id}/stop`),
  uninstall: (id: number) => api.delete(`/apps/${id}`),
}
