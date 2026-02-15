import { Settings as SettingsIcon, User, Lock, Database, Server } from 'lucide-react'

export default function Settings() {
  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-gray-900">设置</h1>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* 账户设置 */}
        <div className="bg-white rounded-xl shadow-sm p-6">
          <div className="flex items-center gap-3 mb-4">
            <User className="w-5 h-5 text-gray-400" />
            <h2 className="text-lg font-semibold">账户设置</h2>
          </div>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                用户名
              </label>
              <input
                type="text"
                className="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                placeholder="用户名"
              />
            </div>
          </div>
        </div>

        {/* 密码设置 */}
        <div className="bg-white rounded-xl shadow-sm p-6">
          <div className="flex items-center gap-3 mb-4">
            <Lock className="w-5 h-5 text-gray-400" />
            <h2 className="text-lg font-semibold">修改密码</h2>
          </div>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                当前密码
              </label>
              <input
                type="password"
                className="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                placeholder="当前密码"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                新密码
              </label>
              <input
                type="password"
                className="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                placeholder="新密码"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                确认新密码
              </label>
              <input
                type="password"
                className="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                placeholder="确认新密码"
              />
            </div>
            <button className="w-full py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700">
              修改密码
            </button>
          </div>
        </div>

        {/* 系统信息 */}
        <div className="bg-white rounded-xl shadow-sm p-6">
          <div className="flex items-center gap-3 mb-4">
            <Server className="w-5 h-5 text-gray-400" />
            <h2 className="text-lg font-semibold">系统信息</h2>
          </div>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between py-2 border-b">
              <span className="text-gray-500">面板版本</span>
              <span className="font-medium">0.1.0</span>
            </div>
            <div className="flex justify-between py-2 border-b">
              <span className="text-gray-500">数据目录</span>
              <span className="font-mono">/opt/panel/data</span>
            </div>
            <div className="flex justify-between py-2 border-b">
              <span className="text-gray-500">配置目录</span>
              <span className="font-mono">/opt/panel/config</span>
            </div>
            <div className="flex justify-between py-2">
              <span className="text-gray-500">监听端口</span>
              <span className="font-mono">3000</span>
            </div>
          </div>
        </div>

        {/* 数据库信息 */}
        <div className="bg-white rounded-xl shadow-sm p-6">
          <div className="flex items-center gap-3 mb-4">
            <Database className="w-5 h-5 text-gray-400" />
            <h2 className="text-lg font-semibold">数据库</h2>
          </div>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between py-2 border-b">
              <span className="text-gray-500">数据库类型</span>
              <span className="font-medium">SQLite</span>
            </div>
            <div className="flex justify-between py-2 border-b">
              <span className="text-gray-500">数据库路径</span>
              <span className="font-mono">/opt/panel/data/panel.db</span>
            </div>
            <div className="flex justify-between py-2">
              <span className="text-gray-500">数据库大小</span>
              <span className="font-mono">-</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
