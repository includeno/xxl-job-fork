import axios from 'axios'

const apiClient = axios.create({
  baseURL: '/api', // This will be proxied to the Rust backend in development
  headers: {
    'Content-Type': 'application/json',
  },
})

export default {
  login(username, password) {
    return apiClient.post('/login', { username, password })
  },
  getDashboardData() {
    return apiClient.get('/dashboard')
  },
  getJobs() {
    return apiClient.get('/job') // Assuming this endpoint will be created to list all jobs
  },
  addJob(jobInfo) {
    return apiClient.post('/job', jobInfo)
  },
  updateJob(jobInfo) {
    return apiClient.put(`/job/${jobInfo.id}`, jobInfo)
  },
  deleteJob(id) {
    return apiClient.delete(`/job/${id}`)
  },
  getJobGroups() {
    return apiClient.get('/job_groups')
  },
  addJobGroup(group) {
    return apiClient.post('/job_group', group)
  },
  updateJobGroup(group) {
    return apiClient.put(`/job_group/${group.id}`, group)
  },
  deleteJobGroup(id) {
    return apiClient.delete(`/job_group/${id}`)
  },
  getJobLogs(jobId) {
    return apiClient.get(`/job/${jobId}/logs`)
  },
  getUsers() {
    return apiClient.get('/users')
  },
  addUser(user) {
    return apiClient.post('/user', user)
  },
  updateUser(user) {
    return apiClient.put(`/user/${user.id}`, user)
  },
  deleteUser(id) {
    return apiClient.delete(`/user/${id}`)
  },
  // Other API functions will be added here
}