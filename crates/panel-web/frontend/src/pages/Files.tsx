import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Folder, File, ChevronRight, RefreshCw, Trash2, Edit, FolderPlus, FilePlus } from 'lucide-react'
import { fileApi } from '../lib/api'
import { formatBytes } from '../lib/utils'

export default function Files() {
  const [currentPath, setCurrentPath] = useState('/')

  const { data, isLoading, refetch } = useQuery({
    queryKey: ['files', currentPath],
    queryFn: () => fileApi.list(currentPath),
  })

  const files = data?.data || []

  const navigateTo = (name: string, isDir: boolean) => {
    if (!isDir) return
    const newPath = currentPath === '/' ? `/${name}` : `${currentPath}/${name}`
    setCurrentPath(newPath)
  }

  const goBack = () => {
    if (currentPath === '/') return
    const parts = currentPath.split('/').filter(Boolean)
    parts.pop()
    setCurrentPath(parts.length === 0 ? '/' : '/' + parts.join('/'))
  }

  const pathParts = currentPath.split('/').filter(Boolean)

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900">文件管理</h1>
        <div className="flex items-center gap-2">
          <button
            onClick={() => refetch()}
            className="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg"
          >
            <RefreshCw className="w-5 h-5" />
          </button>
          <button className="flex items-center gap-2 px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700">
            <FolderPlus className="w-4 h-4" />
            新建文件夹
          </button>
          <button className="flex items-center gap-2 px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700">
            <FilePlus className="w-4 h-4" />
            新建文件
          </button>
        </div>
      </div>

      {/* 路径导航 */}
      <div className="flex items-center gap-1 text-sm bg-white rounded-lg shadow-sm p-3">
        <button
          onClick={() => setCurrentPath('/')}
          className="text-primary-600 hover:underline"
        >
          根目录
        </button>
        {pathParts.map((part, i) => (
          <div key={i} className="flex items-center gap-1">
            <ChevronRight className="w-4 h-4 text-gray-400" />
            <button
              onClick={() => setCurrentPath('/' + pathParts.slice(0, i + 1).join('/'))}
              className="text-primary-600 hover:underline"
            >
              {part}
            </button>
          </div>
        ))}
      </div>

      {/* 文件列表 */}
      <div className="bg-white rounded-xl shadow-sm overflow-hidden">
        {isLoading ? (
          <div className="p-8 text-center text-gray-500">加载中...</div>
        ) : files.length === 0 ? (
          <div className="p-8 text-center text-gray-500">此目录为空</div>
        ) : (
          <table className="w-full">
            <thead>
              <tr className="border-b bg-gray-50">
                <th className="text-left py-3 px-4 font-medium text-gray-500">名称</th>
                <th className="text-right py-3 px-4 font-medium text-gray-500">大小</th>
                <th className="text-right py-3 px-4 font-medium text-gray-500">修改时间</th>
                <th className="text-right py-3 px-4 font-medium text-gray-500">操作</th>
              </tr>
            </thead>
            <tbody>
              {currentPath !== '/' && (
                <tr
                  onClick={goBack}
                  className="border-b hover:bg-gray-50 cursor-pointer"
                >
                  <td className="py-3 px-4">
                    <div className="flex items-center gap-2">
                      <Folder className="w-5 h-5 text-yellow-500" />
                      ..
                    </div>
                  </td>
                  <td className="py-3 px-4 text-right text-gray-500">-</td>
                  <td className="py-3 px-4 text-right text-gray-500">-</td>
                  <td className="py-3 px-4 text-right">-</td>
                </tr>
              )}
              {files.map((file: any, i: number) => (
                <tr
                  key={i}
                  onClick={() => navigateTo(file.name, file.isDir)}
                  className={`border-b last:border-0 hover:bg-gray-50 ${file.isDir ? 'cursor-pointer' : ''}`}
                >
                  <td className="py-3 px-4">
                    <div className="flex items-center gap-2">
                      {file.isDir ? (
                        <Folder className="w-5 h-5 text-yellow-500" />
                      ) : (
                        <File className="w-5 h-5 text-gray-400" />
                      )}
                      {file.name}
                    </div>
                  </td>
                  <td className="py-3 px-4 text-right text-gray-500">
                    {file.isDir ? '-' : formatBytes(file.size)}
                  </td>
                  <td className="py-3 px-4 text-right text-gray-500">
                    {file.modified ? new Date(file.modified).toLocaleString() : '-'}
                  </td>
                  <td className="py-3 px-4 text-right">
                    <div className="flex items-center justify-end gap-2">
                      {!file.isDir && (
                        <button className="p-1 text-gray-400 hover:text-blue-600">
                          <Edit className="w-4 h-4" />
                        </button>
                      )}
                      <button className="p-1 text-gray-400 hover:text-red-600">
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
