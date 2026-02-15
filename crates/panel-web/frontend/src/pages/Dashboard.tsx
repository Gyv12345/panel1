import { useQuery } from '@tanstack/react-query'
import { Cpu, HardDrive, MemoryStick, Network, Activity } from 'lucide-react'
import { systemApi } from '../lib/api'
import { formatBytes, formatUptime } from '../lib/utils'

export default function Dashboard() {
  const { data: systemInfo, isLoading } = useQuery({
    queryKey: ['system', 'info'],
    queryFn: () => systemApi.getInfo(),
    refetchInterval: 5000,
  })

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Activity className="w-8 h-8 animate-spin text-primary-600" />
      </div>
    )
  }

  const info = systemInfo?.data

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-gray-900">仪表盘</h1>

      {/* 系统信息 */}
      <div className="bg-white rounded-xl shadow-sm p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">系统信息</h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div>
            <div className="text-sm text-gray-500">主机名</div>
            <div className="font-medium">{info?.system?.hostname || '-'}</div>
          </div>
          <div>
            <div className="text-sm text-gray-500">操作系统</div>
            <div className="font-medium">{info?.system?.osName || '-'}</div>
          </div>
          <div>
            <div className="text-sm text-gray-500">内核版本</div>
            <div className="font-medium">{info?.system?.kernelVersion || '-'}</div>
          </div>
          <div>
            <div className="text-sm text-gray-500">运行时间</div>
            <div className="font-medium">{formatUptime(info?.system?.uptime || 0)}</div>
          </div>
        </div>
      </div>

      {/* 资源使用 */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* CPU */}
        <div className="bg-white rounded-xl shadow-sm p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-blue-100 rounded-lg">
              <Cpu className="w-6 h-6 text-blue-600" />
            </div>
            <div>
              <div className="text-sm text-gray-500">CPU</div>
              <div className="text-2xl font-bold">{(info?.cpu?.usage || 0).toFixed(1)}%</div>
            </div>
          </div>
          <div className="w-full h-2 bg-gray-200 rounded-full overflow-hidden">
            <div
              className="h-full bg-blue-600 rounded-full transition-all duration-500"
              style={{ width: `${Math.min(info?.cpu?.usage || 0, 100)}%` }}
            />
          </div>
          <div className="mt-2 text-sm text-gray-500">
            {info?.cpu?.cores} 核心 - {info?.cpu?.brand}
          </div>
        </div>

        {/* 内存 */}
        <div className="bg-white rounded-xl shadow-sm p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-green-100 rounded-lg">
              <MemoryStick className="w-6 h-6 text-green-600" />
            </div>
            <div>
              <div className="text-sm text-gray-500">内存</div>
              <div className="text-2xl font-bold">{(info?.memory?.usage || 0).toFixed(1)}%</div>
            </div>
          </div>
          <div className="w-full h-2 bg-gray-200 rounded-full overflow-hidden">
            <div
              className="h-full bg-green-600 rounded-full transition-all duration-500"
              style={{ width: `${Math.min(info?.memory?.usage || 0, 100)}%` }}
            />
          </div>
          <div className="mt-2 text-sm text-gray-500">
            {formatBytes(info?.memory?.used || 0)} / {formatBytes(info?.memory?.total || 0)}
          </div>
        </div>

        {/* 磁盘 */}
        <div className="bg-white rounded-xl shadow-sm p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-purple-100 rounded-lg">
              <HardDrive className="w-6 h-6 text-purple-600" />
            </div>
            <div>
              <div className="text-sm text-gray-500">磁盘</div>
              <div className="text-2xl font-bold">
                {info?.disks?.[0] ? (info.disks[0].usage || 0).toFixed(1) : 0}%
              </div>
            </div>
          </div>
          <div className="w-full h-2 bg-gray-200 rounded-full overflow-hidden">
            <div
              className="h-full bg-purple-600 rounded-full transition-all duration-500"
              style={{ width: `${Math.min(info?.disks?.[0]?.usage || 0, 100)}%` }}
            />
          </div>
          <div className="mt-2 text-sm text-gray-500">
            {formatBytes(info?.disks?.[0]?.used || 0)} / {formatBytes(info?.disks?.[0]?.total || 0)}
          </div>
        </div>
      </div>

      {/* 网络信息 */}
      <div className="bg-white rounded-xl shadow-sm p-6">
        <div className="flex items-center gap-3 mb-4">
          <Network className="w-5 h-5 text-gray-400" />
          <h2 className="text-lg font-semibold text-gray-900">网络接口</h2>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b">
                <th className="text-left py-2 font-medium text-gray-500">接口</th>
                <th className="text-left py-2 font-medium text-gray-500">MAC 地址</th>
                <th className="text-right py-2 font-medium text-gray-500">接收</th>
                <th className="text-right py-2 font-medium text-gray-500">发送</th>
              </tr>
            </thead>
            <tbody>
              {info?.networks?.map((net: any, i: number) => (
                <tr key={i} className="border-b last:border-0">
                  <td className="py-2 font-medium">{net.name}</td>
                  <td className="py-2 text-gray-500">{net.mac}</td>
                  <td className="py-2 text-right">{formatBytes(net.received)}</td>
                  <td className="py-2 text-right">{formatBytes(net.transmitted)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
