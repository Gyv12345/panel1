import { useQuery } from '@tanstack/react-query'
import { AppWindow, Plus, Play, Square, Trash2, RefreshCw, Download } from 'lucide-react'
import { appApi } from '../lib/api'

export default function Apps() {
  const { data: appsData, isLoading: appsLoading } = useQuery({
    queryKey: ['apps'],
    queryFn: () => appApi.list(),
  })

  const { data: templatesData } = useQuery({
    queryKey: ['apps', 'templates'],
    queryFn: () => appApi.templates(),
  })

  const apps = appsData?.data || []
  const templates = templatesData?.data || []

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900">AI 应用管理</h1>
      </div>

      {/* 已安装应用 */}
      <div className="bg-white rounded-xl shadow-sm p-6">
        <h2 className="text-lg font-semibold mb-4">已安装应用</h2>
        {appsLoading ? (
          <div className="text-center text-gray-500 py-4">加载中...</div>
        ) : apps.length === 0 ? (
          <div className="text-center text-gray-500 py-4">暂无已安装的应用</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {apps.map((app: any) => (
              <div key={app.id} className="border rounded-lg p-4 hover:shadow-md transition-shadow">
                <div className="flex items-start justify-between">
                  <div>
                    <h3 className="font-medium">{app.name}</h3>
                    <p className="text-sm text-gray-500">{app.appType}</p>
                  </div>
                  <span className={`px-2 py-1 text-xs rounded-full ${
                    app.status === 'running'
                      ? 'bg-green-100 text-green-800'
                      : 'bg-gray-100 text-gray-800'
                  }`}>
                    {app.status}
                  </span>
                </div>
                {app.port && (
                  <p className="mt-2 text-sm text-gray-500">端口: {app.port}</p>
                )}
                <div className="mt-4 flex items-center gap-2">
                  {app.status === 'running' ? (
                    <button className="flex items-center gap-1 px-3 py-1 text-sm bg-yellow-100 text-yellow-700 rounded hover:bg-yellow-200">
                      <Square className="w-4 h-4" />
                      停止
                    </button>
                  ) : (
                    <button className="flex items-center gap-1 px-3 py-1 text-sm bg-green-100 text-green-700 rounded hover:bg-green-200">
                      <Play className="w-4 h-4" />
                      启动
                    </button>
                  )}
                  <button className="flex items-center gap-1 px-3 py-1 text-sm bg-red-100 text-red-700 rounded hover:bg-red-200">
                    <Trash2 className="w-4 h-4" />
                    卸载
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 应用市场 */}
      <div className="bg-white rounded-xl shadow-sm p-6">
        <h2 className="text-lg font-semibold mb-4">应用市场</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {templates.map((template: any) => (
            <div key={template.appType} className="border rounded-lg p-4 hover:shadow-md transition-shadow">
              <div className="flex items-start justify-between">
                <div>
                  <h3 className="font-medium">{template.name}</h3>
                  <p className="text-sm text-gray-500 mt-1">{template.description}</p>
                </div>
                <AppWindow className="w-8 h-8 text-primary-500" />
              </div>
              <div className="mt-2 text-sm text-gray-500">
                默认端口: {template.defaultPort}
              </div>
              <button className="mt-4 flex items-center gap-1 px-3 py-1.5 text-sm bg-primary-600 text-white rounded hover:bg-primary-700">
                <Download className="w-4 h-4" />
                安装
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
