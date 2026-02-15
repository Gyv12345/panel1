import { useQuery } from '@tanstack/react-query'
import { Globe, Plus, RefreshCw, Lock, Settings, Trash2, ExternalLink } from 'lucide-react'
import { websiteApi } from '../lib/api'

export default function Websites() {
  const { data, isLoading, refetch } = useQuery({
    queryKey: ['websites'],
    queryFn: () => websiteApi.list(),
  })

  const websites = data?.data || []

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900">网站管理</h1>
        <div className="flex items-center gap-2">
          <button
            onClick={() => refetch()}
            className="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg"
          >
            <RefreshCw className="w-5 h-5" />
          </button>
          <button className="flex items-center gap-2 px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700">
            <Plus className="w-4 h-4" />
            添加网站
          </button>
        </div>
      </div>

      <div className="bg-white rounded-xl shadow-sm overflow-hidden">
        {isLoading ? (
          <div className="p-8 text-center text-gray-500">加载中...</div>
        ) : websites.length === 0 ? (
          <div className="p-8 text-center text-gray-500">
            <Globe className="w-12 h-12 mx-auto mb-4 text-gray-300" />
            <p>暂无网站</p>
            <p className="mt-2 text-sm">点击"添加网站"创建你的第一个网站</p>
          </div>
        ) : (
          <table className="w-full">
            <thead>
              <tr className="border-b bg-gray-50">
                <th className="text-left py-3 px-4 font-medium text-gray-500">名称</th>
                <th className="text-left py-3 px-4 font-medium text-gray-500">域名</th>
                <th className="text-left py-3 px-4 font-medium text-gray-500">根目录</th>
                <th className="text-left py-3 px-4 font-medium text-gray-500">状态</th>
                <th className="text-left py-3 px-4 font-medium text-gray-500">SSL</th>
                <th className="text-right py-3 px-4 font-medium text-gray-500">操作</th>
              </tr>
            </thead>
            <tbody>
              {websites.map((site: any) => (
                <tr key={site.id} className="border-b last:border-0 hover:bg-gray-50">
                  <td className="py-3 px-4 font-medium">{site.name}</td>
                  <td className="py-3 px-4">
                    <a
                      href={`http://${site.domain}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-primary-600 hover:underline flex items-center gap-1"
                    >
                      {site.domain}
                      <ExternalLink className="w-3 h-3" />
                    </a>
                  </td>
                  <td className="py-3 px-4 text-gray-500 font-mono text-sm">{site.rootPath}</td>
                  <td className="py-3 px-4">
                    <span className={`px-2 py-1 text-xs rounded-full ${
                      site.status === 'running'
                        ? 'bg-green-100 text-green-800'
                        : 'bg-gray-100 text-gray-800'
                    }`}>
                      {site.status}
                    </span>
                  </td>
                  <td className="py-3 px-4">
                    {site.sslEnabled ? (
                      <span className="flex items-center gap-1 text-green-600">
                        <Lock className="w-4 h-4" />
                        已启用
                      </span>
                    ) : (
                      <span className="text-gray-400">未启用</span>
                    )}
                  </td>
                  <td className="py-3 px-4 text-right">
                    <div className="flex items-center justify-end gap-1">
                      <button className="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded">
                        <Settings className="w-4 h-4" />
                      </button>
                      <button className="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded">
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  )
}
