<template>
  <el-container style="height: 100vh">
    <el-aside width="200px">
      <el-menu :default-openeds="['1']" router>
        <el-sub-menu index="1">
          <template #title>
            <el-icon><setting /></el-icon>Job Management
          </template>
          <el-menu-item index="/jobs">Task Management</el-menu-item>
          <el-menu-item index="/job-groups">Executor Management</el-menu-item>
        </el-sub-menu>
        <el-menu-item index="/logs">
          <el-icon><document /></el-icon>
          <span>Job Logs</span>
        </el-menu-item>
        <el-menu-item index="/users">
          <el-icon><user /></el-icon>
          <span>User Management</span>
        </el-menu-item>
      </el-menu>
    </el-aside>
    <el-main>
      <router-view />
    </el-main>
  </el-container>
</template>

<script setup>
import { Setting, Document, User } from '@element-plus/icons-vue'
</script>

<style scoped>
.el-aside {
  background-color: #545c64;
  color: #fff;
}
.el-menu {
  border-right: none;
}
</style>