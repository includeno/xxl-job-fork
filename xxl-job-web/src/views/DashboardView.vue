<template>
  <div>
    <el-row :gutter="20">
      <el-col :span="6">
        <el-card>
          <el-statistic title="Total Jobs" :value="data.total_jobs" />
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card>
          <el-statistic title="Total Job Groups" :value="data.total_job_groups" />
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card>
          <el-statistic title="Recent Success" :value="data.recent_success_count" />
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card>
          <el-statistic title="Recent Fails" :value="data.recent_fail_count" />
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import api from '@/api'

const data = ref({
  total_jobs: 0,
  total_job_groups: 0,
  recent_success_count: 0,
  recent_fail_count: 0,
})

onMounted(async () => {
  try {
    const response = await api.getDashboardData()
    data.value = response.data
  } catch (error) {
    console.error('Failed to fetch dashboard data:', error)
  }
})
</script>

<style scoped>
.el-card {
  margin-bottom: 20px;
}
</style>