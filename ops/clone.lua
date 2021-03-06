--[[
  Copyright (C) 2011-2021 G. Bajlekov

    Ivy is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Ivy is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
]]

local node = require "ui.node"
local data = require "data"
local thread = require "thread"
local widget = require "ui.widget"

local t = require "ops.tools"

return function(ops)

  local function cloneProcess(self)
  	self.procType = "dev"

  	local i = t.inputSourceBlack(self, 0)
  	local o = t.autoOutputSink(self, 0, i:shape())
  	thread.ops.copy({i, o}, self)

  	local spots = self.widget.getSpots()
  	if #spots>0 then
  		local p = t.autoTempBuffer(self, -1, 1, #spots, 8)
      local ox, oy, fx, fy = widget.imageOffset()
      for k, v in ipairs(spots) do
  			p:set(0, k-1, 0, v.sx*fx-ox)
  			p:set(0, k-1, 1, (1-v.sy)*fy-oy)
  			p:set(0, k-1, 2, v.dx*fx-ox)
  			p:set(0, k-1, 3, (1-v.dy)*fy-oy)
  			p:set(0, k-1, 4, v.size)
  			p:set(0, k-1, 5, v.falloff)
        local r = v.rotation * 2 * math.pi
        p:set(0, k-1, 6, math.cos(r))
        p:set(0, k-1, 7, math.sin(r))
  		end
  		p:syncDev()
  		thread.ops.spotClone({i, o, p}, self)
  	end
  end

  function ops.clone(x, y)
  	local n = node:new("Clone")
  	n:addPortIn(0, "LRGB")
  	n:addPortOut(0, "LRGB")
  	n:addElem("float", 1, "Size", 0, 250, 50)
  	n:addElem("float", 2, "Falloff", 0, 1, 0.5)
    n:addElem("float", 3, "Rotate", -1, 1, 0)

  	n.widget = require "ui.widget.spotmask"(n.elem[1], n.elem[2], n.elem[3])
  	n.widget.toolButton(n, 4, "Manipulate")

  	n.refresh = true
  	n.process = cloneProcess
  	n:setPos(x, y)
  	return n
  end

  local function cloneSmartProcess(self)
  	self.procType = "dev"

  	local i = t.inputSourceBlack(self, 0)
  	local o = t.autoOutputSink(self, 0, i:shape())
  	thread.ops.copy({i, o}, self)

  	local spots = self.widget.getSpots()
  	if #spots>0 then
  		local p = t.autoTempBuffer(self, -1, 1, #spots, 8)
      local ox, oy, fx, fy = widget.imageOffset()
  		for k, v in ipairs(spots) do
        p:set(0, k-1, 0, v.sx*fx-ox)
  			p:set(0, k-1, 1, (1-v.sy)*fy-oy)
  			p:set(0, k-1, 2, v.dx*fx-ox)
  			p:set(0, k-1, 3, (1-v.dy)*fy-oy)
  			p:set(0, k-1, 4, v.size)
  			p:set(0, k-1, 5, v.falloff)
        local r = v.rotation * 2 * math.pi
        p:set(0, k-1, 6, math.cos(r))
        p:set(0, k-1, 7, math.sin(r))
  		end
  		p:syncDev()
  		thread.ops.spotCloneSmart({o, p}, self)
  	end
  end

  function ops.cloneSmart(x, y)
  	local n = node:new("Smart Clone")
  	n:addPortIn(0, "XYZ")
  	n:addPortOut(0, "XYZ")
  	n:addElem("float", 1, "Size", 0, 250, 50)
  	n:addElem("float", 2, "Falloff", 0, 1, 0.5)
    n:addElem("float", 3, "Rotate", -1, 1, 0)

  	n.widget = require "ui.widget.spotmask"(n.elem[1], n.elem[2], n.elem[3])
  	n.widget.toolButton(n, 4, "Manipulate")

  	n.refresh = true
  	n.process = cloneSmartProcess
  	n:setPos(x, y)
  	return n
  end

  local function cloneTextureProcess(self)
  	self.procType = "dev"

  	local i = t.inputSourceBlack(self, 0)
  	local o = t.autoOutputSink(self, 0, i:shape())
  	thread.ops.copy({i, o}, self)

  	local spots = self.widget.getSpots()
  	if #spots>0 then
  		local p = t.autoTempBuffer(self, -1, 1, #spots, 8)
      local ox, oy, fx, fy = widget.imageOffset()
  		for k, v in ipairs(spots) do
        p:set(0, k-1, 0, v.sx*fx-ox)
  			p:set(0, k-1, 1, (1-v.sy)*fy-oy)
  			p:set(0, k-1, 2, v.dx*fx-ox)
  			p:set(0, k-1, 3, (1-v.dy)*fy-oy)
  			p:set(0, k-1, 4, v.size)
  			p:set(0, k-1, 5, v.falloff)
        local r = v.rotation * 2 * math.pi
        p:set(0, k-1, 6, math.cos(r))
        p:set(0, k-1, 7, math.sin(r))
  		end
  		p:syncDev()
  		thread.ops.spotCloneTexture({o, p}, self)
  	end
  end

  function ops.cloneTexture(x, y)
  	local n = node:new("Texture Clone")
  	n:addPortIn(0, "XYZ")
  	n:addPortOut(0, "XYZ")
  	n:addElem("float", 1, "Size", 0, 250, 50)
  	n:addElem("float", 2, "Falloff", 0, 1, 0.5)
    n:addElem("float", 3, "Rotate", -1, 1, 0)

  	n.widget = require "ui.widget.spotmask"(n.elem[1], n.elem[2], n.elem[3])
  	n.widget.toolButton(n, 4, "Manipulate")

  	n.refresh = true
  	n.process = cloneTextureProcess
  	n:setPos(x, y)
  	return n
  end

end
