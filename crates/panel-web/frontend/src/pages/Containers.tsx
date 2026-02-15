import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Container, Play, Square, RotateCw, Trash2, RefreshCw, Download } from 'lucide-react'
import { dockerApi } from '../lib/api'

export default function Containers() {
  const queryClient = useQueryClient()

  const { data, isLoading, refetch } = useQuery({
    queryKey: ['containers'],
    queryFn: () => dockerApi.listContainers(),
  })

  const startMutation = useMutation({
    mutationFn: (id: string) => dockerApi.startContainer(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['containers'] }),
  })

  const stopMutation = useMutation({
    mutationFn: (id: string) => dockerApi.stopContainer(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['containers'] }),
  })

  const restartMutation = useMutation({
    mutationFn: (id: string) => dockerApi.restartContainer(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['containers'] }),
  })

  const removeMutation = useMutation({
    mutationFn: (id: string) => dockerApi.removeContainer(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['containers'] }),
  })

  const containers = data?.data || []

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Running':
        return 'bg-green-100 text-green-800'
      case 'Stopped':
        return 'bg-gray-100 text-gray-800'
      case 'Paused':
        return 'bg-yellow-100 text-yellow-800'
      default:
        return 'bg-gray-100 text-gray-800'
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900">容器管理</h1>
        <div className="flex items-center gap-2">
          <button
            onClick={() => refetch()}
            className="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg"
          >
            <RefreshCw className="w-5 h-5" />
          </button>
          <button className="flex items-center gap-2 px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700">
            <Download className="w-4 h-4" />
            拉取镜像
          </button>
        </div>
      </div>

      <div className="bg-white rounded-xl shadow-sm overflow-hidden">
        {isLoading ? (
          <div className="p-8 text-center text-gray-500">加载中...</div>
        ) : containers.length === 0 ? (
          <div className="p-8 text-center text-gray-500">
            <Container className="w-12 h-12 mx-auto mb-4 text-gray-300" />
            <p>暂无容器</p>
          </div>
        ) : (
          <table className="w-full">
            <thead>
              <tr className="border-b bg-gray-50">
                <th className="text-left py-3 px-4 font-medium text-gray-500">名称</th>
                <th className="text-left py-3 px-4 font-medium text-gray-500">镜像</th>
                <th className="text-left py-3 px-4 font-medium text-gray-500">状态</th>
                <th className="text-left py-3 px-4 font-medium text-gray-500">端口</th>
                <th className="text-right py-3 px-4 font-medium text-gray-500">操作</th>
              </tr>
            </thead>
            <tbody>
              {containers.map((container: any) => (
                <tr key={container.id} className="border-b last:border-0 hover:bg-gray-50">
                  <td className="py-3 px-4">
                    <div className="font-medium">{container.name || container.id.slice(0, 12)}</div>
                  </td>
                  <td className="py-3 px-4 text-gray-500">{container.image}</td>
                  <td className="py-3 px-4">
                    <span className={`px-2 py-1 text-xs rounded-full ${getStatusColor(container.status)}`}>
                      {container.status}
                    </span>
                  </td>
                  <td className="py-3 px-4 text-gray-500">
                    {container.ports?.map((p: any, i: number) => (
                      <span key={i} className="inline-block mr-2">
                        {p.hostIp}:{p.hostPort} → {p.containerPort}
                      </span>
                    ))}
                  </td>
                  <td className="py-3 px-4 text-right">
                    <div className="flex items-center justify-end gap-1">
                      {container.status === 'Running' ? (
                        <button
                          onClick={() => stopMutation.mutate(container.id)}
                          className="p-1.5 text-gray-400 hover:text-yellow-600 hover:bg-yellow-50 rounded"
                          title="停止"
                        >
                          <Square className="w-4 h-4" />
                        </button>
                      ) : (
                        <button
                          onClick={() => startMutation.mutate(container.id)}
                          className="p-1.5 text-gray-400 hover:text-green-600 hover:bg-green-50 rounded"
                          title="启动"
                        >
                          <Play className="w-4 h-4" />
                        </button>
                      )}
                      <button
                        onClick={() => restartMutation.mutate(container.id)}
                        className="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded"
                        title="重启"
                      >
                        <RotateCw className="w-4 h-4" />
                      </button>
                      <button
                        onClick={() => removeMutation.mutate(container.id)}
                        className="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded"
                        title="删除"
                      >
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
